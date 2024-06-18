use p2phandshake::node::Node;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_node_record_valid() {
        let result = Node::parse_node_record("node_id@127.0.0.1:30303");
        assert!(result.is_ok());
        let (addr, port) = result.unwrap();
        assert_eq!(addr, "127.0.0.1");
        assert_eq!(port, 30303);
    }

    #[test]
    fn test_parse_node_record_invalid_format() {
        let result = Node::parse_node_record("node_id127.0.0.1:30303");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_node_record_invalid_port() {
        let result = Node::parse_node_record("node_id@127.0.0.1:notaport");
        assert!(result.is_err());
    }

    #[test]
    fn test_node_initialization() {
        let node_record_str = "node_id@127.0.0.1:30303";
        let node = Node::new(node_record_str).unwrap();
        assert_eq!(node.get_addr(), "127.0.0.1");
        assert_eq!(node.get_port(), 30303);
    }
}
