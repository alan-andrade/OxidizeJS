build:
	rustc --out-dir libs manifest/lib.rs
	rustc --out-dir bin -L libs oxidizejs.rs
