use once::OnceVec;
use fp::prime::*;
use fp::vector::{FpVector, FpVectorT};

pub const MAX_XI_TAU : usize = fp::prime::MAX_MULTINOMIAL_LEN;

// Generated by Mathematica:
// "[\n    " <> # <> "\n]" &[
//  StringJoin @@ 
//   StringReplace[
//    ToString /@ 
//     Riffle[Map[If[# > 2^31, 0, #] &, 
//       Function[p, Function[k, (p^k - 1)/(p - 1)] /@ Range[10]] /@ 
//        Prime[Range[8]], {2}], ",\n    "], {"{" -> "[", "}" -> "]"}]]
static XI_DEGREES : [[i32; 10]; 8] = [
    [1, 3, 7, 15, 31, 63, 127, 255, 511, 1023],
    [1, 4, 13, 40, 121, 364, 1093, 3280, 9841, 29524],
    [1, 6, 31, 156, 781, 3906, 19531, 97656, 488281, 2441406],
    [1, 8, 57, 400, 2801, 19608, 137257, 960800, 6725601, 47079208],
    [1, 12, 133, 1464, 16105, 177156, 1948717, 21435888, 235794769, 0],
    [1, 14, 183, 2380, 30941, 402234, 5229043, 67977560, 883708281, 0],
    [1, 18, 307, 5220, 88741, 1508598, 25646167, 435984840, 0, 0],
    [1, 20, 381, 7240, 137561, 2613660, 49659541, 943531280, 0, 0]
];

// Generated by Mathematica:
// "[\n    " <> # <> "\n]" &[
//  StringJoin @@ 
//   StringReplace[
//    ToString /@ 
//     Riffle[Map[If[# > 2^31, 0, #] &, 
//       Function[p, Function[k, 2 p^k - 1] /@ Range[10]] /@ Prime[Range[8]], {2}], ",\n    "], {"{" -> "[", "}" -> "]"}]]
static TAU_DEGREES : [[i32; 10]; 8] = [
    [1, 3, 7, 15, 31, 63, 127, 255, 511, 1023],
    [1, 5, 17, 53, 161, 485, 1457, 4373, 13121, 39365],
    [1, 9, 49, 249, 1249, 6249, 31249, 156249, 781249, 3906249],
    [1, 13, 97, 685, 4801, 33613, 235297, 1647085, 11529601, 80707213],
    [1, 21, 241, 2661, 29281, 322101, 3543121, 38974341, 428717761, 0],
    [1, 25, 337, 4393, 57121, 742585, 9653617, 125497033, 1631461441, 0],
    [1, 33, 577, 9825, 167041, 2839713, 48275137, 820677345, 0, 0],
    [1, 37, 721, 13717, 260641, 4952197, 94091761, 1787743477, 0, 0]
];

pub fn adem_relation_coefficient(p : ValidPrime, x : u32, y : u32, j : u32, e1 : u32, e2 : u32) -> u32{
    let pi32 = *p as i32;
    let x = x as i32;
    let y = y as i32;
    let j = j as i32;
    let e1 = e1 as i32;
    let e2 = e2 as i32;
    let mut c = binomial(p, (y-j) * (pi32-1) + e1 - 1, x - pi32*j - e2);
    if c == 0 { 
        return 0; 
    }
    c *= minus_one_to_the_n(*p, (x + j) + e2);
    c % *p
}

pub fn inadmissible_pairs(p : ValidPrime, generic : bool, degree : i32) -> Vec<(u32, u32, u32)> {
    let p = *p;
    let degree = degree as u32;
    let q = if generic { 2*p-2 } else { 1 };
    // (i, b, j) means P^i P^j if b = 0, or P^i b P^j if b = 1.
    let mut inadmissible_pairs = Vec::new();

    // Since |P^i| is always a multiple of q, we have a relation only if degree = 0 or 1 mod q.
    // If it is 0, then there is no Bockstein. Otherwise, there is.
    if degree % q == 0 {
        let degq = degree/q;
        // We want P^i P^j to be inadmissible, so i < p * j. This translates to
        // i < p * degq /(p + 1). Since Rust automatically rounds *down*, but we want to round
        // up instead, we use i < (p * degq + p)/(p + 1).
        for i in 1 .. (p * degq + p) / (p + 1) {
            inadmissible_pairs.push((i, 0, degq - i));
        }
    } else if degree % q == 1 {
        let degq = degree/q; // Since we round down, this is actually (degree - 1)/q
        // We want P^i b P^j to be inadmissible, so i < p * j + 1. This translates to
        // i < (p * degq + 1)/(p + 1). Since Rust automatically rounds *down*, but we want to round
        // up instead, we use i < (p * degq + p + 1)/(p + 1).
        for i in 1 .. (p * degq + p + 1) / (p + 1) {
            inadmissible_pairs.push((i, 1, degq - i));
        }
    }
    inadmissible_pairs
}

pub fn tau_degrees(p : ValidPrime) -> &'static [i32] {
    &TAU_DEGREES[PRIME_TO_INDEX_MAP[*p as usize]]
}

pub fn xi_degrees(p : ValidPrime) -> &'static [i32] {
    &XI_DEGREES[PRIME_TO_INDEX_MAP[*p as usize]]
}

struct TruncPolyPartitions {
    p : ValidPrime,
    pub gens : OnceVec<(usize, usize)>, // degree => (first_index, number_of_gens)
    parts : OnceVec<Vec<Vec<FpVector>>> // degree => max_part => list of partitions with maximum part max_part
}

impl TruncPolyPartitions {
    fn new(p : ValidPrime) -> Self {
        let mut gens = OnceVec::new();
        gens.push((0, 0));
        let mut parts = OnceVec::new();
        parts.push(vec![vec![FpVector::new(p, 0)]]);
        Self {
            p,
            gens,
            parts
        }
    }

    fn add_gens_and_calculate_parts(&self, degree : i32, new_gens : usize){
        assert!(degree as usize == self.gens.len());
        let p = *self.p;
        let idx = self.gens[degree as usize - 1].0 + self.gens[degree as usize - 1].1;
        self.gens.push((idx, new_gens));
        let mut new_parts = Vec::new();
        // for i in 0 ..= degree {
        new_parts.push(vec![]);
        // }
        for last_deg in 1 .. degree {
            let mut partitions_cur_max_part = Vec::new();
            let (offset, num_gens) = self.gens[last_deg  as usize];
            let rest_deg = degree - last_deg;
            for (max_part, part_list) in self.parts[rest_deg as usize].iter().enumerate() {
                if max_part > last_deg as usize {
                    break;
                }
                for part in part_list {
                    let mut last_nonzero_entry = usize::max_value();
                    for d in (0 .. num_gens).rev() {
                        let idx = offset + num_gens;
                        if idx > part.dimension() {
                            continue;
                        }
                        if part.entry(d) != 0 {
                            last_nonzero_entry = d;
                            break;
                        }
                    }
                    if last_nonzero_entry > part.dimension() {
                        continue;
                    }
                    if part.entry(last_nonzero_entry) < p-1 {
                        let mut new_part = part.clone();
                        new_part.add_basis_element(last_nonzero_entry, 1);
                        partitions_cur_max_part.push(new_part);
                    }
                    for d in last_nonzero_entry + 1 .. new_gens {
                        let mut new_part = part.clone();
                        new_part.add_basis_element(d, 1);
                        partitions_cur_max_part.push(new_part);
                    }
                }
            }
            new_parts.push(partitions_cur_max_part);
        }
        self.parts.push(new_parts);
    }
}
