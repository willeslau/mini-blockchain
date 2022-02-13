pub use uint::construct_uint;

construct_uint! {
	/// 128-bit unsigned integer.
	pub struct U128(2);
}
construct_uint! {
	/// 256-bit unsigned integer.
	pub struct U256(4);
}
construct_uint! {
	/// 512-bits unsigned integer.
	pub struct U512(8);
}