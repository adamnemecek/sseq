mod chain_homotopy;
mod finite_chain_complex;

use crate::utils::unicode_num;
use algebra::module::homomorphism::{ModuleHomomorphism, MuFreeModuleHomomorphism};
use algebra::module::{Module, MuFreeModule};
use algebra::{Algebra, MuAlgebra};
use bivec::BiVec;
use fp::matrix::Matrix;
use fp::prime::ValidPrime;
use fp::vector::{Slice, SliceMut};
use std::sync::Arc;

use itertools::Itertools;

// pub use hom_complex::HomComplex;
pub use chain_homotopy::ChainHomotopy;
pub use finite_chain_complex::{FiniteAugmentedChainComplex, FiniteChainComplex};

pub enum ChainComplexGrading {
    Homological,
    Cohomological,
}

pub trait FreeChainComplex<const U: bool = false>:
    ChainComplex<
    Module = MuFreeModule<U, <Self as ChainComplex>::Algebra>,
    Homomorphism = MuFreeModuleHomomorphism<U, MuFreeModule<U, <Self as ChainComplex>::Algebra>>,
>
where
    <Self as ChainComplex>::Algebra: MuAlgebra<U>,
{
    fn graded_dimension_string(&self) -> String {
        let mut result = String::new();
        let min_degree = self.min_degree();
        for s in (0..self.next_homological_degree()).rev() {
            let module = self.module(s);

            for t in min_degree + s as i32..=module.max_computed_degree() {
                result.push(unicode_num(module.number_of_gens_in_degree(t)));
                result.push(' ');
            }
            result.push('\n');
            // If it is empty so far, don't print anything
            if result.trim_start().is_empty() {
                result.clear()
            }
        }
        result
    }

    fn to_sseq(&self) -> sseq::Sseq<sseq::Adams> {
        let p = self.prime();
        let mut sseq = sseq::Sseq::new(p, self.min_degree(), 0);
        for (s, n, t) in self.iter_stem() {
            sseq.set_dimension(n, s as i32, self.number_of_gens_in_bidegree(s, t));
        }
        sseq
    }

    fn filtration_one_products(&self, op_deg: i32, op_idx: usize) -> sseq::Product {
        let p = self.prime();
        let mut matrices = BiVec::new(self.min_degree());
        let max_y = self.next_homological_degree() as i32 - 1;
        matrices.extend_with(self.module(0).max_computed_degree() - op_deg + 2, |x| {
            let mut entries = BiVec::with_capacity(0, max_y);
            let mut y = 0;
            while self.has_computed_bidegree(y as u32 + 1, x + y + op_deg) {
                entries.push(
                    self.filtration_one_product(op_deg, op_idx, y as u32, x + y)
                        .map(|m| Matrix::from_vec(p, &m)),
                );
                y += 1;
            }
            entries
        });

        sseq::Product {
            left: true,
            x: op_deg - 1,
            y: 1,
            matrices,
        }
    }

    /// Computes the filtration one product.
    ///
    /// # Returns
    /// If the chain complex is stable, this always returns `Some`. If it is unstable, this returns
    /// `None` if the product is not defined.
    fn filtration_one_product(
        &self,
        op_deg: i32,
        op_idx: usize,
        source_s: u32,
        source_t: i32,
    ) -> Option<Vec<Vec<u32>>> {
        let target_t = source_t + op_deg;
        let target_s = source_s + 1;
        if !self.has_computed_bidegree(target_s, target_t) {
            return None;
        }

        let source = self.module(target_s - 1);
        let target = self.module(target_s);

        if U && op_idx >= self.algebra().dimension_unstable(op_deg, source_t) {
            return None;
        }

        let source_dim = source.number_of_gens_in_degree(source_t);
        let target_dim = target.number_of_gens_in_degree(target_t);

        let d = self.differential(target_s);

        let mut products = vec![Vec::with_capacity(target_dim); source_dim];
        for i in 0..target_dim {
            let dx = d.output(target_t, i);

            for (j, row) in products.iter_mut().enumerate() {
                let idx = source.operation_generator_to_index(op_deg, op_idx, source_t, j);
                row.push(dx.entry(idx));
            }
        }

        Some(products)
    }

    fn number_of_gens_in_bidegree(&self, s: u32, t: i32) -> usize {
        self.module(s).number_of_gens_in_degree(t)
    }

    fn cocycle_string(&self, s: u32, t: i32, idx: usize) -> String {
        let d = self.differential(s);
        let target = d.target();
        let result_vector = d.output(t, idx);

        target.element_to_string_pretty(s, t, result_vector.as_slice())
    }
}

impl<const U: bool, CC> FreeChainComplex<U> for CC
where
    CC: ChainComplex<
        Module = MuFreeModule<U, Self::Algebra>,
        Homomorphism = MuFreeModuleHomomorphism<U, MuFreeModule<U, Self::Algebra>>,
    >,
    Self::Algebra: MuAlgebra<U>,
{
}

/// A chain complex is defined to start in degree 0. The min_degree is the min_degree of the
/// modules in the chain complex, all of which must be the same.
pub trait ChainComplex: Send + Sync {
    type Algebra: Algebra;
    type Module: Module<Algebra = Self::Algebra>;
    type Homomorphism: ModuleHomomorphism<Source = Self::Module, Target = Self::Module>;

    fn prime(&self) -> ValidPrime {
        self.algebra().prime()
    }

    fn algebra(&self) -> Arc<Self::Algebra>;
    fn min_degree(&self) -> i32;
    fn zero_module(&self) -> Arc<Self::Module>;
    fn module(&self, homological_degree: u32) -> Arc<Self::Module>;

    /// This returns the differential starting from the sth module.
    fn differential(&self, s: u32) -> Arc<Self::Homomorphism>;

    /// If the complex has been computed at bidegree (s, t). This means the module has been
    /// computed at (s, t), and so has the differential at (s, t). In the case of a free module,
    /// the target of the differential, namely the bidegree (s - 1, t), need not be computed, as
    /// long as all the generators hit by the differential have already been computed.
    fn has_computed_bidegree(&self, s: u32, t: i32) -> bool;

    /// Ensure all bidegrees less than or equal to (s, t) have been computed
    fn compute_through_bidegree(&self, s: u32, t: i32);

    /// The first s such that `self.module(s)` is not defined.
    fn next_homological_degree(&self) -> u32;

    /// Iterate through all defined bidegrees in increasing order of stem. The return values are of
    /// the form `(s, n, t)`.
    fn iter_stem(&self) -> StemIterator<'_, Self> {
        StemIterator {
            cc: self,
            n: self.min_degree(),
            s: 0,
            max_s: self.next_homological_degree(),
        }
    }

    /// Apply the quasi-inverse of the (s, t)th differential to the list of inputs and results.
    /// This defaults to applying `self.differentials(s).quasi_inverse(t)`, but in some cases
    /// the quasi-inverse might be stored separately on disk.
    ///
    /// This returns whether the application was successful
    #[must_use]
    fn apply_quasi_inverse<T, S>(&self, results: &mut [T], s: u32, t: i32, inputs: &[S]) -> bool
    where
        for<'a> &'a mut T: Into<SliceMut<'a>>,
        for<'a> &'a S: Into<Slice<'a>>,
    {
        assert_eq!(results.len(), inputs.len());
        if results.is_empty() {
            return true;
        }

        let mut iter = inputs.iter().zip_eq(results);
        let (input, result) = iter.next().unwrap();
        let d = self.differential(s);
        if d.apply_quasi_inverse(result.into(), t, input.into()) {
            for (input, result) in iter {
                assert!(d.apply_quasi_inverse(result.into(), t, input.into()));
            }
            true
        } else {
            false
        }
    }

    /// A directory used to save information about the chain complex.
    fn save_dir(&self) -> Option<&std::path::Path> {
        None
    }

    /// Get the save file of a bidegree
    fn save_file(
        &self,
        kind: crate::save::SaveKind,
        s: u32,
        t: i32,
    ) -> crate::save::SaveFile<Self::Algebra> {
        crate::save::SaveFile {
            algebra: self.algebra(),
            kind,
            s,
            t,
            idx: None,
        }
    }
}

/// An iterator returned by [`ChainComplex::iter_stem`]
pub struct StemIterator<'a, CC: ?Sized> {
    cc: &'a CC,
    n: i32,
    s: u32,
    max_s: u32,
}

impl<'a, CC: ChainComplex + ?Sized> Iterator for StemIterator<'a, CC> {
    // (s, n, t)
    type Item = (u32, i32, i32);
    fn next(&mut self) -> Option<Self::Item> {
        if self.max_s == 0 {
            return None;
        }
        let s = self.s;
        let n = self.n;
        let t = self.n + self.s as i32;

        if s == self.max_s {
            self.n += 1;
            self.s = 0;
            return self.next();
        }
        if t > self.cc.module(s).max_computed_degree() {
            if s == 0 {
                return None;
            } else {
                self.n += 1;
                self.s = 0;
                return self.next();
            }
        }
        self.s += 1;
        Some((s, n, t))
    }
}

/// An augmented chain complex is a map of chain complexes C -> D that is a *quasi-isomorphism*. We
/// usually think of C as a resolution of D. The chain map must be a map of degree shift 0.
pub trait AugmentedChainComplex: ChainComplex {
    type TargetComplex: ChainComplex<Algebra = Self::Algebra>;
    type ChainMap: ModuleHomomorphism<
        Source = Self::Module,
        Target = <Self::TargetComplex as ChainComplex>::Module,
    >;

    fn target(&self) -> Arc<Self::TargetComplex>;
    fn chain_map(&self, s: u32) -> Arc<Self::ChainMap>;
}

/// A bounded chain complex is a chain complex C for which C_s = 0 for all s >= max_s
pub trait BoundedChainComplex: ChainComplex {
    fn max_s(&self) -> u32;

    fn euler_characteristic(&self, t: i32) -> isize {
        (0..self.max_s())
            .map(|s| (if s % 2 == 0 { 1 } else { -1 }) * self.module(s).dimension(t) as isize)
            .sum()
    }
}

/// `chain_maps` is required to be non-empty
pub struct ChainMap<F: ModuleHomomorphism> {
    pub s_shift: u32,
    pub chain_maps: Vec<F>,
}
