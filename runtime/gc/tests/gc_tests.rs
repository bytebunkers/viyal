use gc::allocator::GcAllocator;

#[test]
fn test_allocation_and_sweep() {
    let mut allocator = GcAllocator::new();
    let handle1 = allocator.allocate("Hello".to_string());
    let handle2 = allocator.allocate("World".to_string());
    
    // Mark handle 1
    allocator.mark(handle1);
    
    // Sweep should collect handle2
    let collected = allocator.sweep();
    assert_eq!(collected, 1);
    
    // handle1 should still be accessible
    assert_eq!(allocator.get(handle1), Some(&"Hello".to_string()));
    // handle2 should be gone
    assert_eq!(allocator.get(handle2), None);
}
