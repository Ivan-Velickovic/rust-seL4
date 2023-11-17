use core::fmt;

use crate::message::MessageInfo;

pub(crate) type Slot = usize;

pub(crate) const INPUT_CAP: sel4::Endpoint = slot_to_local_cptr(1);
pub(crate) const REPLY_CAP: sel4::Reply = slot_to_local_cptr(4);

const BASE_OUTPUT_NOTIFICATION_CAP: Slot = 10;
const BASE_ENDPOINT_CAP: Slot = BASE_OUTPUT_NOTIFICATION_CAP + 64;
const BASE_IRQ_CAP: Slot = BASE_ENDPOINT_CAP + 64;

const MAX_CHANNELS: Slot = 63;

pub static mut SIGNAL_QUEUED: Option<(sel4::LocalCPtr<sel4::cap_type::Notification>, MessageInfo)> = None;

const fn slot_to_local_cptr<T: sel4::CapType>(slot: Slot) -> sel4::LocalCPtr<T> {
    sel4::LocalCPtr::from_bits(slot as sel4::CPtrBits)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Channel {
    index: usize,
}

impl Channel {
    pub const fn new(index: usize) -> Self {
        assert!(index < MAX_CHANNELS);
        Self { index }
    }

    fn local_cptr<T: sel4::CapType>(&self, offset: Slot) -> sel4::LocalCPtr<T> {
        slot_to_local_cptr(offset + self.index)
    }

    pub fn notify(&self) {
        self.local_cptr::<sel4::cap_type::Notification>(BASE_OUTPUT_NOTIFICATION_CAP)
            .signal()
    }

    pub fn notify_is_queued(&self) -> bool {
        unsafe {
            SIGNAL_QUEUED.is_some()
        }
    }

    pub fn notify_queue(&self) {
        unsafe {
            SIGNAL_QUEUED = Some((
                self.local_cptr::<sel4::cap_type::Notification>(BASE_OUTPUT_NOTIFICATION_CAP),
                MessageInfo::new(0, 0)
            ))
        }
    }

    pub fn irq_ack(&self) -> Result<(), sel4::Error> {
        self.local_cptr::<sel4::cap_type::IRQHandler>(BASE_IRQ_CAP)
            .irq_handler_ack()
    }

    pub fn pp_call(&self, msg_info: MessageInfo) -> MessageInfo {
        MessageInfo::from_sel4(
            self.local_cptr::<sel4::cap_type::Endpoint>(BASE_ENDPOINT_CAP)
                .call(msg_info.into_sel4()),
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct IrqAckError {
    sel4_error: sel4::Error,
}

impl IrqAckError {
    fn as_sel4_error(&self) -> &sel4::Error {
        &self.sel4_error
    }
}

impl fmt::Display for IrqAckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "irq ack error: {:?}", self.as_sel4_error())
    }
}

// // //

// pub const DOES_HAVE_NOTIFICATION_IN: bool = true;
// pub const DOES_HAVE_NOTIFICATION_OUT: bool = true;
// pub const DOES_HAVE_PP_IN: bool = true;
// pub const DOES_HAVE_PP_OUT: bool = true;
// pub const DOES_HAVE_IRQ: bool = true;

// pub struct Channel<
//     const HAS_NOTIFICATION_IN: bool = false,
//     const HAS_NOTIFICATION_OUT: bool = false,
//     const HAS_PP_IN: bool = false,
//     const HAS_PP_OUT: bool = false,
//     const HAS_IRQ: bool = false,
// >(usize);

// impl<
//         const HAS_NOTIFICATION_IN: bool,
//         const HAS_PP_IN: bool,
//         const HAS_PP_OUT: bool,
//         const HAS_IRQ: bool,
//     > Channel<HAS_NOTIFICATION_IN, DOES_HAVE_NOTIFICATION_OUT, HAS_PP_IN, HAS_PP_OUT, HAS_IRQ>
// {
//     pub fn notify(&self) {
//     }
// }
