use crate::*;
use crate::diagram::Diagram;
use std::collections::HashMap;
use std::collections::LinkedList;

pub fn cubicalise(diagram: &DiagramN) {
    log::info!("Diagram was passed to cubicalise {:?}", diagram);
    log::info!("source {:?}", diagram.source().dimension());
    log::info!("target {:?}", diagram.target().dimension());
    log::info!("forward {:?}", diagram.cospans()[0].forward);
    log::info!("backward {:?}", diagram.cospans()[0].backward);

    let d1 = Diagram::rewrite_forward(diagram.source(), &diagram.cospans()[0].forward);
    log::info!("middle {:?}", d1);

    /*let mut node_to_nodes = HashMap::new();
    let list: LinkedList<String> = LinkedList::new();
    node_to_nodes.insert("$", list);

    let n = 1;

    let state = (n, node_to_nodes);*/

}