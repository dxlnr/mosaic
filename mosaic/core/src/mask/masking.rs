//! Masking, aggregation and unmasking of models.
use std::iter::{self, Iterator};

use num::{
    bigint::{BigInt, BigUint, ToBigInt},
    clamp,
    rational::Ratio,
    traits::clamp_max,
};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use thiserror::Error;

use crate::{
    crypto::{prng::generate_integer, ByteObject},
    mask::{
        config::MaskConfigPair,
        object::{MaskObject, MaskUnit, MaskVect},
        scalar::Scalar,
        seed::MaskSeed,
    },
    model::Model,
};

#[derive(Debug, Error, Eq, PartialEq)]
/// Errors related to the unmasking of models.
pub enum UnmaskingError {
    #[error("there is no model to unmask")]
    NoModel,

    #[error("too many models were aggregated for the current unmasking configuration")]
    TooManyModels,

    #[error("too many scalars were aggregated for the current unmasking configuration")]
    TooManyScalars,

    #[error("the masked model is incompatible with the mask used for unmasking")]
    MaskManyMismatch,

    #[error("the masked scalar is incompatible with the mask used for unmasking")]
    MaskOneMismatch,

    #[error("the mask is invalid")]
    InvalidMask,
}

#[derive(Debug, Error)]
/// Errors related to the aggregation of masks and models.
pub enum AggregationError {
    // TODO rename Model -> Vector; or use MaskMany/One terminology
    #[error("the object to aggregate is invalid")]
    InvalidObject,

    #[error("too many models were aggregated for the current unmasking configuration")]
    TooManyModels,

    #[error("too many scalars were aggregated for the current unmasking configuration")]
    TooManyScalars,

    #[error("the model to aggregate is incompatible with the current aggregated scalar")]
    ModelMismatch,

    #[error("the scalar to aggregate is incompatible with the current aggregated scalar")]
    ScalarMismatch,
}

#[derive(Debug, Clone)]
/// An aggregator for masks and masked models.
pub struct Aggregation {
    nb_models: usize,
    object: MaskObject,
    object_size: usize,
}

impl From<MaskObject> for Aggregation {
    fn from(object: MaskObject) -> Self {
        Self {
            nb_models: 1,
            object_size: object.vect.data.len(),
            object,
        }
    }
}

impl From<Aggregation> for MaskObject {
    fn from(aggr: Aggregation) -> Self {
        aggr.object
    }
}

#[allow(clippy::len_without_is_empty)]
impl Aggregation {
    /// Creates a new, empty aggregator for masks or masked models.
    pub fn new(config: MaskConfigPair, object_size: usize) -> Self {
        Self {
            nb_models: 0,
            object: MaskObject::empty(config, object_size),
            object_size,
        }
    }

    /// Gets the length of the aggregated mask object.
    pub fn len(&self) -> usize {
        self.object_size
    }

    /// Gets the masking configurations of the aggregator.
    pub fn config(&self) -> MaskConfigPair {
        MaskConfigPair {
            vect: self.object.vect.config,
            unit: self.object.unit.config,
        }
    }

    /// Validates if unmasking of the aggregated masked model with the given `mask` may be
    /// safely performed.
    ///
    /// This should be checked before calling [`unmask()`], since unmasking may return garbage
    /// values otherwise.
    ///
    /// # Errors
    /// Fails in one of the following cases:
    /// - The aggregator has not yet aggregated any models.
    /// - The number of aggregated masked models is larger than the chosen masking configuration
    ///   allows.
    /// - The masking configuration of the aggregator and of the `mask` don't coincide.
    /// - The length of the aggregated masked model and the `mask` don't coincide.
    /// - The `mask` itself is invalid.
    ///
    /// Even though it does not produce any meaningful values, it is safe and technically possible
    /// due to the [`MaskObject`] type to validate, that:
    /// - a mask may unmask another mask
    /// - a masked model may unmask a mask
    /// - a masked model may unmask another masked model
    ///
    /// [`unmask()`]: Aggregation::unmask
    pub fn validate_unmasking(&self, mask: &MaskObject) -> Result<(), UnmaskingError> {
        // We cannot perform unmasking without at least one real model
        if self.nb_models == 0 {
            return Err(UnmaskingError::NoModel);
        }

        if self.nb_models > self.object.vect.config.model_type.max_nb_models() {
            return Err(UnmaskingError::TooManyModels);
        }

        if self.nb_models > self.object.unit.config.model_type.max_nb_models() {
            return Err(UnmaskingError::TooManyScalars);
        }

        if self.object.vect.config != mask.vect.config || self.object_size != mask.vect.data.len() {
            return Err(UnmaskingError::MaskManyMismatch);
        }

        if self.object.unit.config != mask.unit.config {
            return Err(UnmaskingError::MaskOneMismatch);
        }

        if !mask.is_valid() {
            return Err(UnmaskingError::InvalidMask);
        }

        Ok(())
    }

    /// Unmasks the aggregated masked model with the given `mask`.
    ///
    /// It should be checked that [`validate_unmasking()`] succeeds before calling this, since
    /// unmasking may return garbage values otherwise. The unmasking is performed in opposite order
    /// as described for [`mask()`].
    ///
    /// # Panics
    /// This may only panic if [`validate_unmasking()`] fails.
    ///
    /// Even though it does not produce any meaningful values, it is safe and technically possible
    /// due to the [`MaskObject`] type to unmask:
    /// - a mask with another mask
    /// - a mask with a masked model
    /// - a masked model with another masked model
    ///
    /// if [`validate_unmasking()`] returns `true`.
    ///
    /// [`validate_unmasking()`]: Aggregation::validate_unmasking
    /// [`mask()`]: Masker::mask
    pub fn unmask(self, mask_obj: MaskObject) -> Model {
        let MaskObject { vect, unit } = self.object;
        let (masked_n, config_n) = (vect.data, vect.config);
        let (masked_1, config_1) = (unit.data, unit.config);
        let mask_n = mask_obj.vect.data;
        let mask_1 = mask_obj.unit.data;

        // unmask scalar sum
        let scaled_add_shift_1 = config_1.add_shift() * BigInt::from(self.nb_models);
        let exp_shift_1 = config_1.exp_shift();
        let order_1 = config_1.order();
        let n = (masked_1 + &order_1 - mask_1) % &order_1;
        let ratio = Ratio::<BigInt>::from(n.to_bigint().unwrap());
        let scalar_sum = ratio / &exp_shift_1 - &scaled_add_shift_1;

        // unmask global model
        let scaled_add_shift_n = config_n.add_shift() * BigInt::from(self.nb_models);
        let exp_shift_n = config_n.exp_shift();
        let order_n = config_n.order();
        masked_n
            .into_iter()
            .zip(mask_n)
            .map(|(masked, mask)| {
                // PANIC_SAFE: The substraction panics if it
                // underflows, which can only happen if:
                //
                //     mask > order_n
                //
                // If the mask is valid, we are guaranteed that this
                // cannot happen. Thus this method may panic only if
                // given an invalid mask.
                let n = (masked + &order_n - mask) % &order_n;

                // UNWRAP_SAFE: to_bigint never fails for BigUint
                let ratio = Ratio::<BigInt>::from(n.to_bigint().unwrap());
                let unmasked = ratio / &exp_shift_n - &scaled_add_shift_n;

                // scaling correction
                unmasked / &scalar_sum
            })
            .collect()
    }

    /// Validates if aggregation of the aggregated mask object with the given `object` may be safely
    /// performed.
    ///
    /// This should be checked before calling [`aggregate()`], since aggregation may return garbage
    /// values otherwise.
    ///
    /// # Errors
    /// Fails in one of the following cases:
    /// - The masking configuration of the aggregator and of the `object` don't coincide.
    /// - The length of the aggregated masks or masked model and the `object` don't coincide. If the
    ///   aggregator is empty, then an `object` of any length may be aggregated.
    /// - The new number of aggregated masks or masked models would exceed the number that the
    ///   chosen masking configuration allows.
    /// - The `object` itself is invalid.
    ///
    /// Even though it does not produce any meaningful values, it is safe and technically possible
    /// due to the [`MaskObject`] type to validate, that a mask may be aggregated with a masked
    /// model.
    ///
    /// [`aggregate()`]: Aggregation::aggregate
    pub fn validate_aggregation(&self, object: &MaskObject) -> Result<(), AggregationError> {
        if self.object.vect.config != object.vect.config {
            return Err(AggregationError::ModelMismatch);
        }

        if self.object.unit.config != object.unit.config {
            return Err(AggregationError::ScalarMismatch);
        }

        if self.object_size != object.vect.data.len() {
            return Err(AggregationError::ModelMismatch);
        }

        if self.nb_models >= self.object.vect.config.model_type.max_nb_models() {
            return Err(AggregationError::TooManyModels);
        }

        if self.nb_models >= self.object.unit.config.model_type.max_nb_models() {
            return Err(AggregationError::TooManyScalars);
        }

        if !object.is_valid() {
            return Err(AggregationError::InvalidObject);
        }

        Ok(())
    }

    /// Aggregates the aggregated mask object with the given `object`.
    ///
    /// It should be checked that [`validate_aggregation()`] succeeds before calling this, since
    /// aggregation may return garbage values otherwise.
    ///
    /// # Errors
    /// Even though it does not produce any meaningful values, it is safe and technically possible
    /// due to the [`MaskObject`] type to aggregate a mask with a masked model if
    /// [`validate_aggregation()`] returns `true`.
    ///
    /// [`validate_aggregation()`]: Aggregation::validate_aggregation
    pub fn aggregate(&mut self, object: MaskObject) {
        if self.nb_models == 0 {
            self.object = object.clone();
            self.nb_models = 1;
            return;
        }

        let order_n = self.object.vect.config.order();
        for (i, j) in self
            .object
            .vect
            .data
            .iter_mut()
            .zip(object.vect.data.into_iter())
        {
            *i = (&*i + j) % &order_n
        }

        let order_1 = self.object.unit.config.order();
        let a = &mut self.object.unit.data;
        let b = object.unit.data;
        *a = (&*a + b) % &order_1;

        self.nb_models += 1;
    }
}

/// A masker for models.
pub struct Masker {
    config: MaskConfigPair,
    seed: MaskSeed,
}

impl Masker {
    /// Creates a new masker with the given masking `config`uration with a randomly generated seed.
    pub fn new(config: MaskConfigPair) -> Self {
        Self {
            config,
            seed: MaskSeed::generate(),
        }
    }

    /// Creates a new masker with the given masking `config`uration and `seed`.
    pub fn with_seed(config: MaskConfigPair, seed: MaskSeed) -> Self {
        Self { config, seed }
    }
}

impl Masker {
    /// Masks the given `model` wrt the masking configuration. Enforces bounds on the scalar and
    /// weights.
    ///
    /// The masking proceeds in the following steps:
    /// - Clamp the scalar and the weights according to the masking configuration.
    /// - Scale the weights by the scalar.
    /// - Shift the weights into the non-negative reals.
    /// - Shift the weights into the non-negative integers.
    /// - Shift the weights into the finite group.
    /// - Mask the weights with random elements from the finite group.
    ///
    /// The `scalar` is also masked, following a similar process.
    ///
    /// The random elements are derived from a seeded PRNG. Unmasking as performed in [`unmask()`]
    /// proceeds in reverse order.
    ///
    /// [`unmask()`]: Aggregation::unmask
    pub fn mask(self, scalar: Scalar, model: &Model) -> (MaskSeed, MaskObject) {
        let (random_int, mut random_ints) = self.random_ints();
        let Self { config, seed } = self;
        let MaskConfigPair {
            vect: config_n,
            unit: config_1,
        } = config;

        // clamp the scalar
        let add_shift_1 = config_1.add_shift();
        let scalar_ratio = scalar.into();
        let scalar_clamped = clamp_max(&scalar_ratio, &add_shift_1);

        let exp_shift_n = config_n.exp_shift();
        let add_shift_n = config_n.add_shift();
        let order_n = config_n.order();
        let higher_bound = &add_shift_n;
        let lower_bound = -&add_shift_n;

        // mask the (scaled) weights
        let masked_weights = model
            .iter()
            .zip(&mut random_ints)
            .map(|(weight, rand_int)| {
                let scaled = scalar_clamped * weight;
                let scaled_clamped = clamp(&scaled, &lower_bound, higher_bound);
                // PANIC_SAFE: shifted weight is guaranteed to be non-negative
                let shifted = ((scaled_clamped + &add_shift_n) * &exp_shift_n)
                    .to_integer()
                    .to_biguint()
                    .unwrap();
                (shifted + rand_int) % &order_n
            })
            .collect();
        let masked_model = MaskVect::new_unchecked(config_n, masked_weights);

        // mask the scalar
        // PANIC_SAFE: shifted scalar is guaranteed to be non-negative
        let shifted = ((scalar_clamped + &add_shift_1) * config_1.exp_shift())
            .to_integer()
            .to_biguint()
            .unwrap();
        let masked = (shifted + random_int) % config_1.order();
        let masked_scalar = MaskUnit::new_unchecked(config_1, masked);

        (seed, MaskObject::new_unchecked(masked_model, masked_scalar))
    }

    /// Randomly generates integers wrt the masking configurations.
    ///
    /// The first is generated wrt the scalar configuration, while the rest are
    /// wrt the vector configuration and returned as an iterator.
    fn random_ints(&self) -> (BigUint, impl Iterator<Item = BigUint>) {
        let order_n = self.config.vect.order();
        let order_1 = self.config.unit.order();
        let mut prng = ChaCha20Rng::from_seed(self.seed.as_array());
        let int = generate_integer(&mut prng, &order_1);
        let ints = iter::from_fn(move || Some(generate_integer(&mut prng, &order_n)));
        (int, ints)
    }
}