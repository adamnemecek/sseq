A = AdemAlgebra(2)
A.compute_basis(20)
M = FDModule(A, "M")
M.add_generator(0, "x0")
M.add_generator(1, "x1")
M.parse_action("Sq1 x0 = x1", None)
M.freeze()
r = ext.resolution.Resolver("C2", module=M)
C2 = ResolverChannel(r, REPL)
await C2.setup_a()