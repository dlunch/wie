use alloc::boxed::Box;
use core::{future::Future, marker::PhantomData};

use jvm::JavaValue;

use wie_base::methods;

use crate::JavaContext;

methods!(JavaContext, JavaValue);
