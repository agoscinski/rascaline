use std::os::raw::c_char;
use std::ffi::CStr;
use std::ops::{Deref, DerefMut};

use rascaline::{Calculator, System, CalculationOptions, SelectedIndexes};
use rascaline::descriptor::IndexValue;

use super::utils::copy_str_to_c;
use super::{catch_unwind, rascal_status_t};

use super::descriptor::rascal_descriptor_t;
use super::system::rascal_system_t;

/// Opaque type representing a Calculator
#[allow(non_camel_case_types)]
pub struct rascal_calculator_t(Calculator);

impl Deref for rascal_calculator_t {
    type Target = Calculator;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for rascal_calculator_t {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[no_mangle]
#[allow(clippy::module_name_repetitions)]
pub unsafe extern fn rascal_calculator(name: *const c_char, parameters: *const c_char) -> *mut rascal_calculator_t {
    let mut raw = std::ptr::null_mut();
    let unwind_wrapper = std::panic::AssertUnwindSafe(&mut raw);
    let status = catch_unwind(move || {
        check_pointers!(name, parameters);
        let name = CStr::from_ptr(name).to_str()?;
        let parameters = CStr::from_ptr(parameters).to_str()?;
        let calculator = Calculator::new(name, parameters.to_owned())?;
        let boxed = Box::new(rascal_calculator_t(calculator));

        *unwind_wrapper.0 = Box::into_raw(boxed);
        Ok(())
    });

    if status == rascal_status_t::RASCAL_SUCCESS {
        return raw;
    } else {
        return std::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern fn rascal_calculator_free(calculator: *mut rascal_calculator_t) -> rascal_status_t {
    catch_unwind(|| {
        if !calculator.is_null() {
            let boxed = Box::from_raw(calculator);
            std::mem::drop(boxed);
        }

        Ok(())
    })
}

#[no_mangle]
pub unsafe extern fn rascal_calculator_name(
    calculator: *const rascal_calculator_t,
    name: *mut c_char,
    bufflen: usize
) -> rascal_status_t {
    catch_unwind(|| {
        check_pointers!(calculator, name);
        copy_str_to_c(&(*calculator).name(), name, bufflen)?;
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern fn rascal_calculator_parameters(
    calculator: *const rascal_calculator_t,
    parameters: *mut c_char,
    bufflen: usize
) -> rascal_status_t {
    catch_unwind(|| {
        check_pointers!(calculator, parameters);
        copy_str_to_c(&(*calculator).parameters(), parameters, bufflen)?;
        Ok(())
    })
}

#[repr(C)]
pub struct rascal_calculation_options_t {
    /// Copy the data from systems into native `SimpleSystem`. This can be
    /// faster than having to cross the FFI boundary too often.
    use_native_system: bool,
    /// List of samples on which to run the calculation. Use `NULL` to run the
    /// calculation on all samples.
    selected_samples: *const f64,
    /// If selected_samples is not `NULL`, this should be set to the size of the
    /// selected_samples array
    selected_samples_count: usize,
    /// List of features on which to run the calculation. Use `NULL` to run the
    /// calculation on all features.
    selected_features: *const f64,
    /// If selected_features is not `NULL`, this should be set to the size of the
    /// selected_features array
    selected_features_count: usize,
}

impl<'a> From<&'a rascal_calculation_options_t> for CalculationOptions<'a> {
    fn from(options: &'a rascal_calculation_options_t) -> CalculationOptions {
        let selected_samples = if options.selected_samples.is_null() {
            SelectedIndexes::All
        } else {
            let slice = unsafe {
                std::slice::from_raw_parts(
                    options.selected_samples as *const IndexValue,
                    options.selected_samples_count
                )
            };
            SelectedIndexes::FromC(slice)
        };

        let selected_features = if options.selected_features.is_null() {
            SelectedIndexes::All
        } else {
            let slice = unsafe {
                std::slice::from_raw_parts(
                    options.selected_features as *const IndexValue,
                    options.selected_features_count
                )
            };
            SelectedIndexes::FromC(slice)
        };

        CalculationOptions {
            use_native_system: options.use_native_system,
            selected_samples: selected_samples,
            selected_features: selected_features,
        }
    }
}

#[no_mangle]
pub unsafe extern fn rascal_calculator_compute(
    calculator: *mut rascal_calculator_t,
    descriptor: *mut rascal_descriptor_t,
    systems: *mut rascal_system_t,
    systems_count: usize,
    options: rascal_calculation_options_t,
) -> rascal_status_t {
    catch_unwind(|| {
        if systems_count == 0 {
            // TODO: warning
            return Ok(());
        }
        check_pointers!(calculator, descriptor, systems);

        // Create a Vec<&mut dyn System> from the passed systems
        let systems = std::slice::from_raw_parts_mut(systems, systems_count);
        let mut references = Vec::new();
        for system in systems {
            references.push(system as &mut dyn System);
        }

        let options = (&options).into();
        (*calculator).compute(&mut references, &mut *descriptor, options)
    })
}
