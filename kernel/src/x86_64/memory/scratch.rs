// pub unsafe fn scratch_paging_info() {
//     let cr3: u64 = read_cr3();

//     println!("CR3: {:016X}", cr3);

//     let cr3_address = HIGHER_HALF_MASK | (cr3);
//     let test_address = HIGHER_HALF_MASK | 0x00100000;
//     println!("PML4 BASE: {:016X}", cr3_address);

//     let (pml4_index, pdpt_index, pdt_index, pt_index) = get_page_indexes_from_address(test_address);
//     let test = *(test_address as *const u64);
//     let test = *((HIGHER_HALF_MASK | 0x00808000) as *const u64);
//     print_page_indexes_from_address(test_address);
//     entries_for_table_with_values(cr3_address);
//     println!();

//     let pml4_entry = *((cr3_address as *const u64).add(pml4_index as usize));
//     let pdpt_address = (pml4_entry & PAGE_ENTRY_PHYSICAL_MASK) | HIGHER_HALF_MASK;
//     println!("PDPT BASE: {:016X}", pdpt_address);
//     // print_page_indexes_from_address(pdpt_address);
//     entries_for_table_with_values(pdpt_address);
//     println!();

//     let pdpt_entry = *((pdpt_address as *const u64).add(pdpt_index as usize));
//     let pdt_address = (pdpt_entry & PAGE_ENTRY_PHYSICAL_MASK) | HIGHER_HALF_MASK;
//     println!("PDT BASE: {:016X}", pdt_address);
//     // print_page_indexes_from_address(pdt_address);
//     entries_for_table_with_values(pdt_address);
//     // println!();

//     let pdt_entry = *((pdt_address as *const u64).add(pdt_index as usize));
//     let pt_address = (pdt_entry & PAGE_ENTRY_PHYSICAL_MASK) | HIGHER_HALF_MASK;
//     println!("PT BASE: {:016X}", pt_address);
//     // print_page_indexes_from_address(pt_address);
//     // entries_for_table_with_values(pt_address);
//     // println!();

//     let pt_entry = *((pt_address as *const u64).add(pt_index as usize));
//     println!("PT ENTRY: {:016X}", pt_entry);
// }

// //7FE0300
// // 0x0010 0000
