#![no_std]
#![no_main]
#![feature(never_type)]

use core::ptr::NonNull;

use virtio_drivers::{
    device::net::*,
    transport::{
        mmio::{MmioTransport, VirtIOHeader},
        DeviceType, Transport,
    },
};

use sel4_externally_shared::ExternallySharedRef;
use sel4_microkit::{memory_region_symbol, protection_domain, var, Channel, Handler, MessageInfo};
use sel4_microkit_message::MessageInfoExt as _;
use sel4_shared_ring_buffer::{RingBuffer, RingBuffers};

use microkit_http_server_example_virtio_hal_impl::HalImpl;
use microkit_http_server_example_virtio_net_driver_interface_types::*;

const DEVICE: Channel = Channel::new(0);
const CLIENT: Channel = Channel::new(1);

const NET_QUEUE_SIZE: usize = 16;
const NET_BUFFER_LEN: usize = 2048;

#[protection_domain(
    heap_size = 512 * 1024,
)]
fn init() -> HandlerImpl {
    HalImpl::init(
        *var!(virtio_net_driver_dma_size: usize = 0),
        *var!(virtio_net_driver_dma_vaddr: usize = 0),
        *var!(virtio_net_driver_dma_paddr: usize = 0),
    );

    let mut dev = {
        let header = NonNull::new(
            (*var!(virtio_net_mmio_vaddr: usize = 0) + *var!(virtio_net_mmio_offset: usize = 0))
                as *mut VirtIOHeader,
        )
        .unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();
        assert_eq!(transport.device_type(), DeviceType::Network);
        VirtIONet::<HalImpl, MmioTransport, NET_QUEUE_SIZE>::new(transport, NET_BUFFER_LEN).unwrap()
    };

    let client_region = unsafe {
        ExternallySharedRef::<'static, _>::new(
            memory_region_symbol!(virtio_net_client_dma_vaddr: *mut [u8], n = *var!(virtio_net_client_dma_size: usize = 0)),
        )
    };

    let client_client_dma_region_paddr = *var!(virtio_net_client_dma_paddr: usize = 0);

    let rx_ring_buffers = unsafe {
        RingBuffers::<'_, fn() -> Result<(), !>>::new(
            RingBuffer::from_ptr(memory_region_symbol!(virtio_net_rx_free: *mut _)),
            RingBuffer::from_ptr(memory_region_symbol!(virtio_net_rx_used: *mut _)),
            notify_client,
            true,
        )
    };

    let tx_ring_buffers = unsafe {
        RingBuffers::<'_, fn() -> Result<(), !>>::new(
            RingBuffer::from_ptr(memory_region_symbol!(virtio_net_tx_free: *mut _)),
            RingBuffer::from_ptr(memory_region_symbol!(virtio_net_tx_used: *mut _)),
            notify_client,
            true,
        )
    };

    dev.ack_interrupt();
    DEVICE.irq_ack().unwrap();

    HandlerImpl {
        dev,
        client_region,
        client_client_dma_region_paddr,
        rx_ring_buffers,
        tx_ring_buffers,
    }
}

fn notify_client() -> Result<(), !> {
    CLIENT.notify();
    Ok::<_, !>(())
}

struct HandlerImpl {
    dev: VirtIONet<HalImpl, MmioTransport, NET_QUEUE_SIZE>,
    client_region: ExternallySharedRef<'static, [u8]>,
    client_client_dma_region_paddr: usize,
    rx_ring_buffers: RingBuffers<'static, fn() -> Result<(), !>>,
    tx_ring_buffers: RingBuffers<'static, fn() -> Result<(), !>>,
}

impl Handler for HandlerImpl {
    type Error = !;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        match channel {
            DEVICE | CLIENT => {
                let mut notify_rx = false;

                while self.dev.can_recv() && !self.rx_ring_buffers.free().is_empty() {
                    let rx_buf = self.dev.receive().unwrap();
                    let desc = self.rx_ring_buffers.free_mut().dequeue().unwrap();
                    let desc_len = usize::try_from(desc.len()).unwrap();
                    assert!(desc_len >= rx_buf.packet_len());
                    let buf_range = {
                        let start = desc.encoded_addr() - self.client_client_dma_region_paddr;
                        start..start + rx_buf.packet_len()
                    };
                    self.client_region
                        .as_mut_ptr()
                        .index(buf_range)
                        .copy_from_slice(rx_buf.packet());
                    self.dev.recycle_rx_buffer(rx_buf).unwrap();
                    self.rx_ring_buffers.used_mut().enqueue(desc).unwrap();
                    notify_rx = true;
                }

                if notify_rx {
                    self.rx_ring_buffers.notify().unwrap();
                }

                let mut notify_tx = false;

                while !self.tx_ring_buffers.free().is_empty() && self.dev.can_send() {
                    let desc = self.tx_ring_buffers.free_mut().dequeue().unwrap();
                    let buf_range = {
                        let start = desc.encoded_addr() - self.client_client_dma_region_paddr;
                        start..start + usize::try_from(desc.len()).unwrap()
                    };
                    let mut tx_buf = self.dev.new_tx_buffer(buf_range.len());
                    self.client_region
                        .as_ptr()
                        .index(buf_range)
                        .copy_into_slice(tx_buf.packet_mut());
                    self.dev.send(tx_buf).unwrap();
                    self.tx_ring_buffers.used_mut().enqueue(desc).unwrap();
                    notify_tx = true;
                }

                if notify_tx {
                    self.tx_ring_buffers.notify().unwrap();
                }

                self.dev.ack_interrupt();
                DEVICE.irq_ack().unwrap();
            }
            _ => {
                unreachable!()
            }
        }
        Ok(())
    }

    fn protected(
        &mut self,
        channel: Channel,
        msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        Ok(match channel {
            CLIENT => match msg_info.recv_using_postcard::<Request>() {
                Ok(req) => match req {
                    Request::GetMacAddress => {
                        let mac_address = self.dev.mac_address();
                        MessageInfo::send_using_postcard(GetMacAddressResponse {
                            mac_address: MacAddress(mac_address),
                        })
                        .unwrap()
                    }
                },
                Err(_) => MessageInfo::send_unspecified_error(),
            },
            _ => {
                unreachable!()
            }
        })
    }
}
