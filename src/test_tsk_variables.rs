// These tests basically make sure that we
// can actually bind these things

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_node_is_sample() {
        let x = bindings::TSK_NODE_IS_SAMPLE;
        assert!(x > 0);
    }
}
