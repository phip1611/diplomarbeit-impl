#[cfg(test)]
mod tests {
    use elf_rs::Elf;

    #[test]
    fn test_parse_elf() {
        let elf_bytes = include_bytes!("../../../../build/fileserver-bin_debug_stripped.elf");
        let elf = elf_rs::Elf::from_bytes(elf_bytes).unwrap();
        let elf64 = match elf {
            Elf::Elf64(elf) => elf,
            _ => panic!("unexpected elf 32"),
        };
        let pr_hdrs = elf64.program_header_iter().collect::<Vec<_>>();
        dbg!(pr_hdrs);
    }
}
