use nazar_core::windows_installer::packed_product_code;

#[test]
fn product_code_is_packed_the_way_windows_installer_stores_it() {
    let packed = packed_product_code("{12345678-ABCD-EF01-2345-6789abcdef01}");

    assert_eq!(packed.as_deref(), Some("87654321DCBA10FE32547698BADCFE10"));
}

#[test]
fn key_name_that_is_not_a_product_code_is_not_packed() {
    assert_eq!(packed_product_code("WinRAR archiver"), None);
    assert_eq!(packed_product_code("{not-a-product-code}"), None);
    assert_eq!(
        packed_product_code("{12345678-ABCD-EF01-2345-6789ABCDEF01}_is1"),
        None
    );
}
