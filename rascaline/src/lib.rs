#![warn(clippy::all, clippy::pedantic)]

// disable some style lints
#![allow(clippy::needless_return, clippy::missing_errors_doc, clippy::must_use_candidate)]
#![allow(clippy::redundant_field_names, clippy::redundant_closure_for_method_calls)]
#![allow(clippy::unreadable_literal, clippy::option_if_let_else, clippy::range_plus_one)]
#![allow(clippy::comparison_chain)]

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_wrap, clippy::cast_lossless, clippy::cast_sign_loss)]
#![allow(clippy::default_trait_access)]

// Tests lints
#![cfg_attr(test, allow(clippy::float_cmp))]

pub mod types;
pub use types::*;

pub(crate) mod math;

mod errors;
pub use self::errors::Error;

pub mod system;
pub use system::{System, SimpleSystem};

pub mod descriptor;
pub use descriptor::Descriptor;

mod calculator;
pub use calculator::{Calculator, CalculationOptions, SelectedIndexes};

pub mod calculators;
