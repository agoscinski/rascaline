use std::collections::BTreeMap;

use crate::{SimpleSystem, descriptor::{Descriptor, Indexes, IndexValue, IndexesBuilder}};
use crate::system::System;
use crate::Error;

use crate::calculators::CalculatorBase;

pub struct Calculator {
    implementation: Box<dyn CalculatorBase>,
    parameters: String,
}

/// List of pre-selected indexes on which the user wants to run a calculation
pub enum SelectedIndexes<'a> {
    /// Default, all indexes
    All,
    /// Only the list of selected indexes
    Some(Indexes),
    /// Internal use: list of selected indexes as passed through the C API
    #[doc(hidden)]
    FromC(&'a [IndexValue]),
}

impl<'a> SelectedIndexes<'a> {
    fn into_features(self, calculator: &dyn CalculatorBase) -> Result<Indexes, Error> {
        let indexes = match self {
            SelectedIndexes::All => calculator.features(),
            SelectedIndexes::Some(indexes) => indexes,
            SelectedIndexes::FromC(list) => {
                let mut builder = IndexesBuilder::new(calculator.features_names());

                if list.len() % builder.size() != 0 {
                    return Err(Error::InvalidParameter(format!(
                        "wrong size for partial features list, expected a multiple of {}, got {}",
                        builder.size(), list.len()
                    )))
                }

                for chunk in list.chunks(builder.size()) {
                    builder.add(chunk);
                }
                builder.finish()
            }
        };

        calculator.check_features(&indexes);
        return Ok(indexes);
    }

    fn into_samples(
        self,
        calculator: &dyn CalculatorBase,
        systems: &mut [&mut dyn System],
    ) -> Result<Indexes, Error> {
        let indexes = match self {
            SelectedIndexes::All => {
                let environments = calculator.environments();
                environments.indexes(systems)
            },
            SelectedIndexes::Some(indexes) => indexes,
            SelectedIndexes::FromC(list) => {
                let environments = calculator.environments();
                let mut builder = IndexesBuilder::new(environments.names());

                if list.len() % builder.size() != 0 {
                    return Err(Error::InvalidParameter(format!(
                        "wrong size for partial samples list, expected a multiple of {}, got {}",
                        builder.size(), list.len()
                    )))
                }

                for chunk in list.chunks(builder.size()) {
                    builder.add(chunk);
                }
                builder.finish()
            }
        };

        calculator.check_environments(&indexes, systems);
        return Ok(indexes);
    }
}

/// Parameters specific to a single call to `compute`
pub struct CalculationOptions<'a> {
    /// Copy the data from systems into native `SimpleSystem`. This can be
    /// faster than having to cross the FFI boundary too often.
    pub use_native_system: bool,
    /// List of selected samples on which to run the computation
    pub selected_samples: SelectedIndexes<'a>,
    /// List of selected features on which to run the computation
    pub selected_features: SelectedIndexes<'a>,
}

impl<'a> Default for CalculationOptions<'a> {
    fn default() -> CalculationOptions<'a> {
        CalculationOptions {
            use_native_system: false,
            selected_samples: SelectedIndexes::All,
            selected_features: SelectedIndexes::All,
        }
    }
}

impl From<Box<dyn CalculatorBase>> for Calculator {
    fn from(implementation: Box<dyn CalculatorBase>) -> Calculator {
        let parameters = implementation.get_parameters();
        Calculator {
            implementation: implementation,
            parameters: parameters,
        }
    }
}

impl Calculator {
    pub fn new(name: &str, parameters: String) -> Result<Calculator, Error> {
        let creator = match REGISTERED_CALCULATORS.get(name) {
            Some(creator) => creator,
            None => {
                return Err(Error::InvalidParameter(
                    format!("unknown calculator with name '{}'", name)
                ));
            }
        };

        return Ok(Calculator {
            implementation: creator(&parameters)?,
            parameters: parameters,
        })
    }

    /// Get the name associated with this Calculator
    pub fn name(&self) -> String {
        self.implementation.name()
    }

    /// Get the parameters used to create this Calculator in a string.
    ///
    /// Currently the string is formatted as JSON, but this could change in the
    /// future.
    pub fn parameters(&self) -> &str {
        &self.parameters
    }

    /// Compute the descriptor for all the given `systems` and store it in
    /// `descriptor`
    ///
    /// This function computes the full descriptor, using all samples and all
    /// features.
    pub fn compute(
        &mut self,
        systems: &mut [&mut dyn System],
        descriptor: &mut Descriptor,
        options: CalculationOptions,
    ) -> Result<(), Error> {
        let features = options.selected_features.into_features(&*self.implementation)?;
        let samples = options.selected_samples.into_samples(&*self.implementation, systems)?;

        let environments_builder = self.implementation.environments();
        if self.implementation.compute_gradients() {
            let gradients = environments_builder
                .gradients_for(systems, &samples)
                .expect("this environments definition do not support gradients");
            descriptor.prepare_gradients(samples, gradients, features);
        } else {
            descriptor.prepare(samples, features);
        }

        if options.use_native_system {
            let mut native_systems = to_native_systems(systems);
            let mut references = Vec::with_capacity(systems.len());
            for system in &mut native_systems {
                references.push(system as &mut dyn System);
            }

            self.implementation.compute(&mut references, descriptor);
        } else {
            self.implementation.compute(systems, descriptor);
        }

        return Ok(());
    }
}

fn to_native_systems(systems: &mut [&mut dyn System]) -> Vec<SimpleSystem> {
    let mut native_systems = Vec::with_capacity(systems.len());
    for system in systems.iter() {
        native_systems.push(SimpleSystem::from(*system as &dyn System));
    }
    return native_systems;
}

/// Registration of calculator implementations
use crate::calculators::{DummyCalculator, SortedDistances};
use crate::calculators::{SphericalExpansion, SphericalExpansionParameters};
type CalculatorCreator = fn(&str) -> Result<Box<dyn CalculatorBase>, Error>;

macro_rules! add_calculator {
    ($map :expr, $name :literal, $type :ty) => (
        $map.insert($name, (|json| {
            let value = serde_json::from_str::<$type>(json)?;
            Ok(Box::new(value))
        }) as CalculatorCreator);
    );
    ($map :expr, $name :literal, $type :ty, $parameters :ty) => (
        $map.insert($name, (|json| {
            let parameters = serde_json::from_str::<$parameters>(json)?;
            Ok(Box::new(<$type>::new(parameters)))
        }) as CalculatorCreator);
    );
}

lazy_static::lazy_static!{
    pub static ref REGISTERED_CALCULATORS: BTreeMap<&'static str, CalculatorCreator> = {
        let mut map = BTreeMap::new();
        add_calculator!(map, "dummy_calculator", DummyCalculator);
        add_calculator!(map, "sorted_distances", SortedDistances);
        add_calculator!(map, "spherical_expansion", SphericalExpansion, SphericalExpansionParameters);
        return map;
    };
}
