use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, HtmlElement, Node, NodeList, Window};

#[allow(dead_code)]
pub async fn get_document(window: &Window) -> Option<Document> {
    window.document()
}

#[allow(dead_code)]
pub async fn query_node_list(window: &Window, query: &str) -> Result<NodeList, JsValue> {
    get_document(window)
        .await
        .expect("window returned no Document! Is a page loaded?")
        .query_selector_all(query)
}

#[allow(dead_code)]
pub async fn query_node(window: &Window, query: &str, offset: u32) -> Option<Node> {
    query_node_list(window, query)
        .await
        .expect("query_node_list failed or returned no results!")
        .get(offset)
}

#[allow(dead_code)]
pub async fn query_node_first(window: &Window, query: &str) -> Option<Node> {
    query_node(window, query, 0).await
}

#[allow(dead_code)]
pub async fn node_to_html_element(node: Node) -> Result<HtmlElement, Node> {
    node.dyn_into()
}

#[allow(dead_code)]
pub async fn query_html_element(
    window: &Window,
    query: &str,
    offset: u32,
) -> Result<HtmlElement, Node> {
    node_to_html_element(
        query_node(window, query, offset)
            .await
            .expect("query_node failed or returned no results!"),
    )
    .await
}

#[allow(dead_code)]
pub async fn query_html_element_first(window: &Window, query: &str) -> Result<HtmlElement, Node> {
    query_html_element(window, query, 0).await
}
