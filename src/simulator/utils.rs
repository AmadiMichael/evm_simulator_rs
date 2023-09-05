use ethers::types::{Address, H256, U256};

pub fn u64_array_to_u8_array(input: [u64; 4]) -> [u8; 32] {
    let mut output = [0; 32];

    for (i, &u64_value) in input.iter().enumerate() {
        let bytes = u64_value.swap_bytes().to_le_bytes();

        let u = 3 - i;

        output[u * 8..(u + 1) * 8].copy_from_slice(&bytes);
    }

    output
}

pub fn u256_to_address(input: U256) -> Address {
    Address::from(H256::from(u64_array_to_u8_array(input.0)))
}

pub fn write_to_output_file<T: std::fmt::Debug>(to_write: &T) {
    // Specify the file path you want to write to
    let file_path: &str = "output.txt";

    // Open the file for writing (creates the file if it doesn't exist)
    let mut file = std::fs::File::create(file_path).expect("failed to create file");

    let st = format!("{:?}", to_write);

    // Write the data to the file
    std::io::Write::write_all(&mut file, st.as_bytes()).expect("failed to write to created file");
}
