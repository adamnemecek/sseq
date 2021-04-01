use parking_lot::Mutex;
use std::sync::Arc;

use crate::chain_complex::{AugmentedChainComplex, ChainComplex};
use algebra::module::homomorphism::{FreeModuleHomomorphism, ModuleHomomorphism};
use algebra::module::{FreeModule, Module};
use algebra::Algebra;
use fp::matrix::{AugmentedMatrix3, Subspace};
use fp::prime::ValidPrime;
use fp::vector::FpVector;
use once::{OnceBiVec, OnceVec};

#[cfg(feature = "concurrent")]
use crossbeam_channel::{unbounded, Receiver};

#[cfg(feature = "concurrent")]
use thread_token::TokenBucket;

/// A resolution of a chain complex.
pub struct Resolution<CC: ChainComplex> {
    lock: Mutex<()>,
    complex: Arc<CC>,
    modules: OnceVec<Arc<FreeModule<<CC::Module as Module>::Algebra>>>,
    zero_module: Arc<FreeModule<<CC::Module as Module>::Algebra>>,
    chain_maps: OnceVec<Arc<FreeModuleHomomorphism<CC::Module>>>,
    differentials:
        OnceVec<Arc<FreeModuleHomomorphism<FreeModule<<CC::Module as Module>::Algebra>>>>,

    ///  For each *internal* degree, store the kernel of the most recently calculated chain map as
    ///  returned by `generate_old_kernel_and_compute_new_kernel`, to be used if we run
    ///  resolve_through_degree again.
    kernels: OnceBiVec<Mutex<Option<Subspace>>>,
}

impl<CC: ChainComplex> Resolution<CC> {
    pub fn new(complex: Arc<CC>) -> Self {
        let algebra = complex.algebra();
        let min_degree = complex.min_degree();
        let zero_module = Arc::new(FreeModule::new(
            Arc::clone(&algebra),
            "F_{-1}".to_string(),
            min_degree,
        ));

        Self {
            complex,
            zero_module,
            lock: Mutex::new(()),

            chain_maps: OnceVec::new(),
            modules: OnceVec::new(),
            differentials: OnceVec::new(),
            kernels: OnceBiVec::new(min_degree),
        }
    }

    pub fn extended_degree(&self) -> (u32, i32) {
        (self.modules.len() as u32, self.kernels.len())
    }

    /// This function prepares the Resolution object to perform computations up to the
    /// specified s degree. It does *not* perform any computations by itself. It simply lengthens
    /// the `OnceVec`s `modules`, `chain_maps`, etc. to the right length.
    fn extend_through_degree(&self, max_s: u32, max_t: i32) {
        let min_degree = self.min_degree();

        for i in self.modules.len() as u32..=max_s {
            self.modules.push(Arc::new(FreeModule::new(
                Arc::clone(&self.algebra()),
                format!("F{}", i),
                min_degree,
            )));
            self.chain_maps.push(Arc::new(FreeModuleHomomorphism::new(
                Arc::clone(&self.modules[i]),
                Arc::clone(&self.complex.module(i)),
                0,
            )));
        }

        for _ in self.kernels.len() as i32..=max_t {
            self.kernels.push(Mutex::new(None));
        }

        if self.differentials.is_empty() {
            self.differentials
                .push(Arc::new(FreeModuleHomomorphism::new(
                    Arc::clone(&self.modules[0u32]),
                    Arc::clone(&self.zero_module),
                    0,
                )));
        }

        for i in self.differentials.len() as u32..=max_s {
            self.differentials
                .push(Arc::new(FreeModuleHomomorphism::new(
                    Arc::clone(&self.modules[i]),
                    Arc::clone(&self.modules[i - 1]),
                    0,
                )));
        }
    }

    /// Call our resolution $X$, and the chain complex to resolve $C$. This is a legitimate
    /// resolution if the map $f: X \to C$ induces an isomorphism on homology. This is the same as
    /// saying the cofiber is exact. The cofiber is given by the complex
    ///
    /// $$ X_s \oplus C_{s+1} \to X_{s-1} \oplus C_s \to X_{s-2} \oplus C_{s-1} \to \cdots $$
    ///
    /// where the differentials are given by
    ///
    /// $$ \begin{pmatrix} d_X & 0 \\\\ (-1)^s f & d_C \end{pmatrix} $$
    ///
    /// Our method of producing $X_{s, t}$ and the chain maps are as follows. Suppose we have already
    /// built the chain map and differential for $X_{s-1, t}$ and $X_{s, t-1}$. Since $X_s$ is a
    /// free module, the generators in degree $< t$ gives us a bunch of elements in $X_s$ already,
    /// and we know exactly where they get mapped to. Let $T$ be the $\\mathbb{F}_p$ vector space
    /// generated by these elements. Then we already have a map
    ///
    /// $$ T \to X_{s-1, t} \oplus C_{s, t}$$
    ///
    /// and we know this hits the kernel of the map
    ///
    /// $$ D = X_{s-1, t} \oplus C_{s, t} \to X_{s-2, t} \oplus C_{s-1, t}. $$
    ///
    /// What we need to do now is to add generators to $X_{s, t}$ to hit the entirity of this
    /// kernel.  Note that we don't *have* to do this. Some of the elements in the kernel might be
    /// hit by $C_{s+1, t}$ and we don't have to hit them, but we opt to add generators to hit it
    /// anyway.
    ///
    /// If we do it this way, then we know the composite of the map
    ///
    /// $$ T \to X_{s-1, t} \oplus C_{s, t} \to C_{s, t} $$
    ///
    /// has to be surjective, since the image of $C_{s, t}$ under $D$ is also in the image of $X_{s-1, t}$.
    /// So our first step is to add generators to $X_{s, t}$ such that this composite is
    /// surjective.
    ///
    /// After adding these generators, we need to decide where to send them to. We know their
    /// values in the $C_{s, t}$ component, but we need to use a quasi-inverse to find the element in
    /// $X_{s-1, t}$ that hits the corresponding image of $C_{s, t}$. This tells us the $X_{s-1,
    /// t}$ component.
    ///
    /// Finally, we need to add further generators to $X_{s, t}$ to hit all the elements in the
    /// kernel of
    ///
    /// $$ X_{s-1, t} \to X_{s-2, t} \oplus C_{s-1, t}. $$
    ///
    /// This kernel was recorded by the previous iteration of the method in `old_kernel`, so this
    /// step is doable as well.
    ///
    /// Note that if we add our new generators conservatively, then the kernel of the maps
    ///
    /// $$
    /// \begin{aligned}
    /// T &\to X_{s-1, t} \oplus C_{s, t} \\\\
    /// X_{s, t} &\to X_{s-1, t} \oplus C_{s, t}
    /// \end{aligned}
    /// $$
    /// agree.
    ///
    /// In the code, we first row reduce the matrix of the map from $T$. This lets us record
    /// the kernel which is what the function returns at the end. This computation helps us perform
    /// the future steps since we need to know about the cokernel of this map.
    ///
    /// # Arguments
    ///  * `s` - The s degree to calculate
    ///  * `t` - The t degree to calculate
    pub fn step_resolution(&self, s: u32, t: i32) {
        if s == 0 {
            self.zero_module.extend_by_zero(t);
        }

        let mut old_kernel = self.kernels[t].lock();
        let p = self.prime();

        //                           current_chain_map
        //                X_{s, t} --------------------> C_{s, t}
        //                   |                               |
        //                   | current_differential          |
        //                   v                               v
        // old_kernel <= X_{s-1, t} -------------------> C_{s-1, t}

        let complex = self.complex();
        complex.compute_through_bidegree(s, t);

        let current_differential = self.differential(s);
        let current_chain_map = self.chain_map(s);
        let complex_cur_differential = complex.differential(s);

        match current_differential.next_degree().cmp(&t) {
            std::cmp::Ordering::Greater => {
                // Already computed this degree.
                return;
            }
            std::cmp::Ordering::Less => {
                // Haven't computed far enough yet
                panic!("We're not ready to compute bidegree ({}, {}) yet.", s, t);
            }
            std::cmp::Ordering::Equal => (),
        };

        let source = self.module(s);
        let target_cc = complex.module(s);
        let target_res = current_differential.target(); // This is self.module(s - 1) unless s = 0.

        source.extend_table_entries(t);

        let chain_map_lock = current_chain_map.lock();
        let differential_lock = current_differential.lock();

        // The Homomorphism matrix has size source_dimension x target_dimension, but we are going to augment it with an
        // identity matrix so that gives a matrix with dimensions source_dimension x (target_dimension + source_dimension).
        // Later we're going to write into this same matrix an isomorphism source/image + new vectors --> kernel
        // This has size target_dimension x (2*target_dimension).
        // This latter matrix may be used to find a preimage of an element under the differential.
        let source_dimension = source.dimension(t);
        let target_cc_dimension = target_cc.dimension(t);
        let target_res_dimension = target_res.dimension(t);

        let rows = source_dimension + target_cc_dimension + target_res_dimension;

        let mut matrix = AugmentedMatrix3::new(
            p,
            rows,
            &[
                target_cc_dimension,
                target_res_dimension,
                source_dimension + rows,
            ],
        );
        // Get the map (d, f) : X_{s, t} -> X_{s-1, t} (+) C_{s, t} into matrix

        current_chain_map.get_matrix(&mut matrix.segment(0, 0).row_slice(0, source_dimension), t);
        current_differential
            .get_matrix(&mut matrix.segment(1, 1).row_slice(0, source_dimension), t);
        matrix.segment(2, 2).add_identity(source_dimension, 0, 0);
        matrix.initialize_pivots();

        // This slices the underling matrix. Be sure to revert this.
        let matrix_start_2 = matrix.start[2];
        let mut pivots = matrix.take_pivots();
        matrix
            .slice_mut(0, source_dimension, 0, matrix_start_2 + source_dimension)
            .row_reduce_into_pivots(&mut pivots);
        let new_kernel = matrix
            .slice_mut(0, source_dimension, 0, matrix_start_2 + source_dimension)
            .compute_kernel(&pivots, matrix_start_2);
        matrix.set_pivots(pivots);

        let first_new_row = source_dimension;

        // Now add generators to surject onto C_{s, t}.
        // (For now we are just adding the eventual images of the new generators into matrix, we will update
        // X_{s,t} and f later).
        // We record which pivots exactly we added so that we can walk over the added genrators in a moment and
        // work out what dX should to to each of them.
        let new_generators = matrix
            .inner
            .extend_to_surjection(first_new_row, 0, matrix.end[0]);
        let cc_new_gens = new_generators.len();

        let mut res_new_gens = 0;

        let mut middle_rows = Vec::with_capacity(cc_new_gens);
        if s > 0 {
            if cc_new_gens > 0 {
                // Now we need to make sure that we have a chain homomorphism. Each generator x we just added to
                // X_{s,t} has a nontrivial image f(x) \in C_{s,t}. We need to set d(x) so that f(dX(x)) = dC(f(x)).
                // So we set dX(x) = f^{-1}(dC(f(x)))
                let prev_chain_map = self.chain_map(s - 1);
                let quasi_inverse = prev_chain_map.quasi_inverse(t);

                let dfx_dim = complex_cur_differential.target().dimension(t);
                let mut dfx = FpVector::new(self.prime(), dfx_dim);

                for (i, column) in new_generators.into_iter().enumerate() {
                    complex_cur_differential.apply_to_basis_element(
                        dfx.as_slice_mut(),
                        1,
                        t,
                        column,
                    );
                    quasi_inverse.apply(
                        matrix.row_segment(first_new_row + i, 1, 1),
                        1,
                        dfx.as_slice(),
                    );
                    dfx.set_to_zero();

                    // Keep the rows we produced because we have to row reduce to re-compute
                    // the kernel later, but these rows are the images of the generators, so we
                    // still need them.
                    middle_rows.push(matrix[first_new_row + i].clone());
                }
                // Row reduce again since our activity may have changed the image of dX.
                matrix.row_reduce();
            }
            // Now we add new generators to hit any cycles in old_kernel that we don't want in our homology.
            res_new_gens = matrix
                .inner
                .extend_image(
                    first_new_row + cc_new_gens,
                    matrix.start[1],
                    matrix.end[1],
                    old_kernel.as_ref(),
                )
                .len();

            if cc_new_gens > 0 {
                // Now restore the middle rows.
                for (i, row) in middle_rows.into_iter().enumerate() {
                    matrix[first_new_row + i] = row;
                }
            }
        }
        let num_new_gens = cc_new_gens + res_new_gens;
        source.add_generators(t, num_new_gens, None);

        current_chain_map.add_generators_from_matrix_rows(
            &chain_map_lock,
            t,
            matrix.segment(0, 0).row_slice(first_new_row, rows),
        );
        current_differential.add_generators_from_matrix_rows(
            &differential_lock,
            t,
            matrix.segment(1, 1).row_slice(first_new_row, rows),
        );

        // Record the quasi-inverses for future use.
        // The part of the matrix that contains interesting information is occupied_rows x (target_dimension + source_dimension + kernel_size).
        let image_rows = first_new_row + num_new_gens;
        for i in first_new_row..image_rows {
            matrix.inner[i].set_entry(matrix_start_2 + i, 1);
        }

        // From now on we only use the underlying matrix.
        let mut pivots = matrix.take_pivots();
        matrix
            .slice_mut(
                0,
                image_rows,
                0,
                matrix_start_2 + source_dimension + num_new_gens,
            )
            .row_reduce_into_pivots(&mut pivots);
        matrix.set_pivots(pivots);

        let (cm_qi, res_qi) =
            matrix.compute_quasi_inverses(matrix_start_2 + source_dimension + num_new_gens);

        current_chain_map.set_quasi_inverse(&chain_map_lock, t, cm_qi);
        current_chain_map.set_kernel(&chain_map_lock, t, Subspace::new(p, 0, 0)); // Fill it up with something dummy so that compute_kernels_and... is happy
        current_differential.set_quasi_inverse(&differential_lock, t, res_qi);
        current_differential.set_kernel(&differential_lock, t, Subspace::new(p, 0, 0));

        *old_kernel = Some(new_kernel);
    }

    // pub fn step_resolution_by_stem(&self, s : u32, t : i32) {
    //     // println!("\n\n\n\n");
    //     // println!("s: {}, t: {} || x: {}, y: {}", s, t, t-s as i32, s);
    //     // println!("s: {}, t: {} || x: {}, y: {}", s, t, t-s as i32, s);
    //     if s == 0 {
    //         self.zero_module.extend_by_zero(t);
    //     }

    //     let p = self.prime();

    //     //                           current_chain_map
    //     //                X_{s, t} --------------------> C_{s, t}
    //     //                   |                               |
    //     //                   | current_differential          |
    //     //                   v                               v
    //     // old_kernel <= X_{s-1, t} -------------------> C_{s-1, t}

    //     let complex = self.complex();
    //     complex.compute_through_bidegree(s, t + 1);

    //     let current_differential = self.differential(s);
    //     let current_chain_map = self.chain_map(s);
    //     let complex_cur_differential = complex.differential(s);

    //     match current_differential.next_degree().cmp(&t) {
    //         std::cmp::Ordering::Greater => {
    //             // Already computed this degree.
    //             return;
    //         }
    //         std::cmp::Ordering::Less => {
    //             // Haven't computed far enough yet
    //             panic!("We need to compute bidegree ({}, {}) before we are ready to compute bidegree ({}, {}).", s, t-1, s, t);
    //         }
    //         std::cmp::Ordering::Equal => ()
    //     };

    //     if s > 0 && self.differential(s-1).next_degree() < t - 1 {
    //         panic!("We need to compute bidegree ({}, {}) before we are ready to compute bidegree ({}, {}).", s-1, t-1, s, t);
    //     }

    //     let source = self.module(s);
    //     let target_cc = complex.module(s);
    //     let target_res = current_differential.target(); // This is self.module(s - 1) unless s = 0.
    //     source.extend_table_entries(t+1);
    //     target_res.extend_table_entries(t+1);

    //     let chain_map_lock = current_chain_map.lock();
    //     let differential_lock = current_differential.lock();

    //     // The Homomorphism matrix has size source_dimension x target_dimension, but we are going to augment it with an
    //     // identity matrix so that gives a matrix with dimensions source_dimension x (target_dimension + source_dimension).
    //     // Later we're going to write into this same matrix an isomorphism source/image + new vectors --> kernel
    //     // This has size target_dimension x (2*target_dimension).
    //     // This latter matrix may be used to find a preimage of an element under the differential.
    //     let target_cc_dimension = target_cc.dimension(t);
    //     let target_res_dimension = target_res.dimension(t);
    //     let source_dimension = source.dimension(t);
    //     let rows = target_cc_dimension + target_res_dimension + source_dimension;

    //     // Calculate how many pivots are missing / gens to add
    //     let kernel = self.kernels[s][t].lock().take();
    //     let maybe_image = self.images[s][t].lock().take();
    //     let mut image : Image;
    //     // let old_rows;
    //     if let Some(x) = maybe_image {
    //         image = x;
    //         // old_rows = image.matrix.segment(2,2).columns();
    //         image.resize_target_res_dimension(target_res_dimension);
    //     } else {
    //         image = Image {
    //             matrix : AugmentedMatrix3::new(p, rows, &[target_cc_dimension, target_res_dimension, rows]),
    //             pivots : vec![-1; target_cc_dimension + target_res_dimension + rows ]
    //         };
    //         // old_rows = rows;
    //         image.matrix.segment(2, 2).set_identity(rows, 0, 0);
    //     }

    //     let matrix = &mut image.matrix;
    //     let pivots = &mut image.pivots;

    //     // Now add generators to surject onto C_{s, t}.
    //     // (For now we are just adding the eventual images of the new generators into matrix, we will update
    //     // X_{s,t} and f later).
    //     // We record which pivots exactly we added so that we can walk over the added generators in a moment and
    //     // work out what dX should to to each of them.
    //     let first_new_row = source_dimension;
    //     let new_generators = matrix.inner.extend_to_surjection(first_new_row, 0, target_cc_dimension, &pivots);
    //     let cc_new_gens = new_generators.len();
    //     let mut res_new_gens = 0;

    //     let mut middle_rows = Vec::with_capacity(cc_new_gens);
    //     if s > 0 {
    //         if cc_new_gens > 0 {
    //             // Now we need to make sure that we have a chain homomorphism. Each generator x we just added to
    //             // X_{s,t} has a nontrivial image f(x) \in C_{s,t}. We need to set d(x) so that f(dX(x)) = dC(f(x)).
    //             // So we set dX(x) = f^{-1}(dC(f(x)))
    //             let prev_chain_map = self.chain_map(s - 1);
    //             let quasi_inverse = prev_chain_map.quasi_inverse(t);

    //             let dfx_dim = complex_cur_differential.target().dimension(t);
    //             let mut dfx = FpVector::new(self.prime(), dfx_dim);

    //             for (i, column) in new_generators.into_iter().enumerate() {
    //                 complex_cur_differential.apply_to_basis_element(&mut dfx, 1, t, column);
    //                 quasi_inverse.apply(&mut *matrix.row_segment(first_new_row + i, 1, 1), 1, &dfx);
    //                 dfx.set_to_zero();

    //                 // Keep the rows we produced because we have to row reduce to re-compute
    //                 // the kernel later, but these rows are the images of the generators, so we
    //                 // still need them.
    //                 middle_rows.push(matrix[first_new_row + i].clone());
    //             }
    //             // Row reduce again since our activity may have changed the image of dX.
    //             matrix.row_reduce(pivots);
    //         }
    //         // println!("matrix.seg(1) : {}", *matrix.segment(1,1));
    //         // Now we add new generators to hit any cycles in old_kernel that we don't want in our homology.
    //         res_new_gens = matrix.inner.extend_image(
    //             first_new_row + cc_new_gens,
    //             matrix.start[1], matrix.end[1],
    //             pivots, kernel.as_ref()
    //         ).len();

    //         if cc_new_gens > 0 {
    //             // Now restore the middle rows.
    //             for (i, row) in middle_rows.into_iter().enumerate() {
    //                 matrix[first_new_row + i] = row;
    //             }
    //         }
    //     }

    //     // println!("cc_new_gens : {}, res_new_gens: {}", cc_new_gens, res_new_gens);
    //     let num_new_gens = cc_new_gens + res_new_gens;
    //     source.add_generators(t, num_new_gens, None);

    //     let rows = matrix.rows();
    //     matrix.set_row_slice(first_new_row, rows);
    //     current_chain_map.add_generators_from_matrix_rows(&chain_map_lock, t, &*matrix.segment(0, 0));
    //     current_differential.add_generators_from_matrix_rows(&differential_lock, t, &*matrix.segment(1, 1));
    //     matrix.clear_row_slice();

    //     // Record the quasi-inverses for future use.
    //     // The part of the matrix that contains interesting information is occupied_rows x (target_dimension + source_dimension + kernel_size).
    //     let image_rows = first_new_row + num_new_gens;
    //     for i in first_new_row .. image_rows {
    //         matrix.inner[i].set_entry(matrix.start[2] + i, 1);
    //     }

    //     // From now on we only use the underlying matrix. We manipulate slice directly but don't
    //     // drop matrix so that we can use matrix.start
    //     matrix.inner.set_slice(0, image_rows, 0, matrix.start[2] + source_dimension + num_new_gens);
    //     let mut new_pivots = vec![-1;matrix.columns()];
    //     matrix.row_reduce(&mut new_pivots);

    //     // Should this be a method on AugmentedMatrix3?
    //     let (cm_qi, res_qi) = matrix.compute_quasi_inverses(&new_pivots);

    //     current_chain_map.set_quasi_inverse(&chain_map_lock, t, cm_qi);
    //     current_chain_map.set_kernel(&chain_map_lock, t, Subspace::new(p, 0, 0)); // Fill it up with something dummy so that compute_kernels_and... is happy
    //     current_differential.set_quasi_inverse(&differential_lock, t, res_qi);
    //     current_differential.set_kernel(&differential_lock, t, Subspace::new(p, 0, 0));

    //     let target_cc_dimension = target_cc.dimension(t+1);
    //     let target_res_dimension = target_res.dimension(t+1);
    //     let source_dimension = source.dimension(t+1);
    //     target_res.extend_table_entries(t+1);
    //     source.extend_table_entries(t+1);

    //     // Now we are going to investigate the homomorphism in degree t + 1.

    //     // Now need to calculate new_kernel and new_image.

    //     let rows = source_dimension + target_cc_dimension + target_res_dimension;
    //     let mut matrix = AugmentedMatrix3::new(p, rows, &[target_cc_dimension, target_res_dimension, rows]);
    //     let mut pivots = vec![-1;matrix.columns()];
    //     // Get the map (d, f) : X_{s, t} -> X_{s-1, t} (+) C_{s, t} into matrix

    //     matrix.set_row_slice(0, source_dimension);
    //     current_chain_map.get_matrix(&mut *matrix.segment(0,0), t + 1);
    //     current_differential.get_matrix(&mut *matrix.segment(1,1), t + 1);
    //     matrix.segment(2,2).set_identity(rows, 0, 0);

    //     matrix.row_reduce(&mut pivots);
    //     let new_kernel = matrix.inner.compute_kernel(&pivots, matrix.start[2]);

    //     let mut kernel_lock = self.kernels[s + 1][t+1].lock();
    //     *kernel_lock = Some(new_kernel);
    //     if s > 0 {
    //         let mut image_lock = self.images[s][t + 1].lock();
    //         *image_lock = Some(Image {
    //             matrix : matrix,
    //             pivots : pivots
    //         });
    //         drop(image_lock);
    //     }
    //     drop(kernel_lock);

    // }

    pub fn cocycle_string(&self, hom_deg: u32, int_deg: i32, idx: usize) -> String {
        let d = self.differential(hom_deg);
        let target = d.target();
        let result_vector = d.output(int_deg, idx);

        target.element_to_string_pretty(hom_deg, int_deg, result_vector.as_slice())
    }

    pub fn complex(&self) -> Arc<CC> {
        Arc::clone(&self.complex)
    }

    pub fn number_of_gens_in_bidegree(&self, s: u32, t: i32) -> usize {
        self.module(s).number_of_gens_in_degree(t)
    }

    pub fn prime(&self) -> ValidPrime {
        self.complex.prime()
    }

    #[cfg(feature = "concurrent")]
    pub fn resolve_through_bidegree_concurrent(
        &self,
        max_s: u32,
        max_t: i32,
        bucket: &TokenBucket,
    ) {
        self.resolve_through_bidegree_concurrent_with_callback(max_s, max_t, bucket, |_, _| ())
    }

    pub fn resolve_through_bidegree(&self, max_s: u32, max_t: i32) {
        self.resolve_through_bidegree_with_callback(max_s, max_t, |_, _| ())
    }

    #[cfg(feature = "concurrent")]
    pub fn resolve_through_bidegree_concurrent_with_callback(
        &self,
        max_s: u32,
        max_t: i32,
        bucket: &TokenBucket,
        mut cb: impl FnMut(u32, i32),
    ) {
        let min_degree = self.min_degree();
        let _lock = self.lock.lock();

        self.complex().compute_through_bidegree(max_s, max_t);
        self.extend_through_degree(max_s, max_t);
        self.algebra().compute_basis(max_t - min_degree);

        crossbeam_utils::thread::scope(|s| {
            let (pp_sender, pp_receiver) = unbounded();
            let mut last_receiver: Option<Receiver<()>> = None;
            for t in min_degree..=max_t {
                let (sender, receiver) = unbounded();

                let pp_sender = pp_sender.clone();
                s.spawn(move |_| {
                    let mut token = bucket.take_token();
                    for s in 0..=max_s {
                        token = bucket.recv_or_release(token, &last_receiver);
                        if !self.has_computed_bidegree(s, t) {
                            self.step_resolution(s, t);

                            pp_sender.send((s, t)).unwrap();
                        }
                        sender.send(()).unwrap();
                    }
                });
                last_receiver = Some(receiver);
            }
            // We drop this pp_sender, so that when all previous threads end, no pp_sender's are
            // present, so pp_receiver terminates.
            drop(pp_sender);

            for (s, t) in pp_receiver {
                cb(s, t);
            }
        })
        .unwrap();
    }

    pub fn resolve_through_bidegree_with_callback(
        &self,
        max_s: u32,
        max_t: i32,
        mut cb: impl FnMut(u32, i32),
    ) {
        let min_degree = self.min_degree();
        let _lock = self.lock.lock();

        self.complex().compute_through_bidegree(max_s, max_t);
        self.extend_through_degree(max_s, max_t);
        self.algebra().compute_basis(max_t - min_degree);

        for t in min_degree..=max_t {
            for s in 0..=max_s {
                if self.has_computed_bidegree(s, t) {
                    continue;
                }
                self.step_resolution(s, t);
                cb(s, t);
            }
        }
    }
}

impl<CC: ChainComplex> ChainComplex for Resolution<CC> {
    type Algebra = CC::Algebra;
    type Module = FreeModule<Self::Algebra>;
    type Homomorphism = FreeModuleHomomorphism<FreeModule<Self::Algebra>>;

    fn algebra(&self) -> Arc<Self::Algebra> {
        self.complex().algebra()
    }

    fn module(&self, s: u32) -> Arc<Self::Module> {
        Arc::clone(&self.modules[s as usize])
    }

    fn zero_module(&self) -> Arc<Self::Module> {
        Arc::clone(&self.zero_module)
    }

    fn min_degree(&self) -> i32 {
        self.complex().min_degree()
    }

    fn has_computed_bidegree(&self, s: u32, t: i32) -> bool {
        self.differentials.len() > s as usize && self.differential(s).next_degree() > t
    }

    fn set_homology_basis(&self, _s: u32, _t: i32, _homology_basis: Vec<usize>) {
        unimplemented!()
    }

    fn homology_basis(&self, _s: u32, _t: i32) -> &Vec<usize> {
        unimplemented!()
    }

    fn homology_dimension(&self, s: u32, t: i32) -> usize {
        self.number_of_gens_in_bidegree(s, t)
    }

    fn max_homology_degree(&self, _s: u32) -> i32 {
        unimplemented!()
    }

    fn differential(&self, s: u32) -> Arc<Self::Homomorphism> {
        Arc::clone(&self.differentials[s as usize])
    }

    fn compute_through_bidegree(&self, s: u32, t: i32) {
        assert!(self.has_computed_bidegree(s, t));
    }

    fn max_homological_degree(&self) -> u32 {
        self.modules.len() as u32 - 1
    }
}

impl<CC: ChainComplex> AugmentedChainComplex for Resolution<CC> {
    type TargetComplex = CC;
    type ChainMap = FreeModuleHomomorphism<CC::Module>;

    fn target(&self) -> Arc<Self::TargetComplex> {
        self.complex()
    }

    fn chain_map(&self, s: u32) -> Arc<Self::ChainMap> {
        Arc::clone(&self.chain_maps[s])
    }
}

use saveload::{Load, Save};
use std::io;
use std::io::{Read, Write};

impl<CC: ChainComplex> Save for Resolution<CC> {
    fn save(&self, buffer: &mut impl Write) -> io::Result<()> {
        let max_algebra_dim = self.module(0).max_computed_degree() - self.min_degree();

        max_algebra_dim.save(buffer)?;
        self.modules.save(buffer)?;
        self.kernels.save(buffer)?;
        self.differentials.save(buffer)?;
        self.chain_maps.save(buffer)?;
        Ok(())
    }
}

impl<CC: ChainComplex> Load for Resolution<CC> {
    type AuxData = Arc<CC>;

    fn load(buffer: &mut impl Read, cc: &Self::AuxData) -> io::Result<Self> {
        let max_algebra_dim = i32::load(buffer, &())?;
        cc.algebra().compute_basis(max_algebra_dim);

        let mut result = Resolution::new(Arc::clone(cc));

        let algebra = result.algebra();
        let p = result.prime();
        let min_degree = result.min_degree();

        result.modules = Load::load(buffer, &(Arc::clone(&algebra), min_degree))?;
        result.kernels = Load::load(buffer, &(min_degree, Some(p)))?;

        let max_s = result.modules.len();
        assert!(max_s > 0, "cannot load uninitialized resolution");

        let len = usize::load(buffer, &())?;
        assert_eq!(len, max_s);

        result.differentials.push(Load::load(
            buffer,
            &(result.module(0), result.zero_module(), 0),
        )?);
        for s in 1..max_s as u32 {
            let d: Arc<FreeModuleHomomorphism<FreeModule<CC::Algebra>>> =
                Load::load(buffer, &(result.module(s), result.module(s - 1), 0))?;
            result.differentials.push(d);
        }

        let len = usize::load(buffer, &())?;
        assert_eq!(len, max_s);

        for s in 0..max_s as u32 {
            let c: Arc<FreeModuleHomomorphism<CC::Module>> =
                Load::load(buffer, &(result.module(s), result.complex().module(s), 0))?;
            result.chain_maps.push(c);
        }

        result
            .zero_module
            .extend_by_zero(result.module(0).max_computed_degree());

        Ok(result)
    }
}
