use alloc::boxed::Box;
use core::{future::Future, marker::PhantomData};

use wie_base::methods;

use crate::{JavaContext, JavaWord};

methods!(JavaContext, JavaWord);
