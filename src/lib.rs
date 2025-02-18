pub mod ipc_client;
pub mod json_parser;

pub fn run() {
    ipc_client::run_ipc();
    println!("ran");
}

pub fn split_workspaces(ipc_output: &str) -> Vec<String> {
    let mut stack: Vec<u8> = Vec::new();

    let mut workspaces: Vec<String> = Vec::new();
    let mut workspace_string: String = String::new();

    for ipc_char in ipc_output.chars() {
        workspace_string.push(ipc_char);
        match ipc_char {
            '{' => stack.push(1),
            '}' => stack.truncate(stack.len() - 1),
            ',' => continue,
            ' ' => continue,
            _ => {}
        }
        if stack.is_empty() {
            workspaces.push(workspace_string);
            workspace_string = String::new();
        }
    }

    workspaces
}

pub fn get_num_workspaces(output: &str) -> i32 {
    let workspaces = split_workspaces(output);
    let _: Vec<_> = workspaces
        .iter()
        .map(|workspace| println!("{workspace:?}"))
        .collect();
    workspaces.len() as i32
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_num_workspaces_simple() {
        let output: String = String::from("{ \"id\": 4, \"type\": \"workspace\", \"orientation\": \"horizontal\", \"percent\": null, \"urgent\": false, \"marks\": [ ], \"layout\": \"splith\", \"border\": \"none\", \"current_border_width\": 0, \"rect\": { \"x\": 0, \"y\": 0, \"width\": 1920, \"height\": 1080 }, \"deco_rect\": { \"x\": 0, \"y\": 0, \"width\": 0, \"height\": 0 }, \"window_rect\": { \"x\": 0, \"y\": 0, \"width\": 0, \"height\": 0 }, \"geometry\": { \"x\": 0, \"y\": 0, \"width\": 0, \"height\": 0 }, \"name\": \"1\", \"window\": null, \"nodes\": [ ], \"floating_nodes\": [ ], \"focus\": [ 8 ], \"fullscreen_mode\": 1, \"sticky\": false, \"floating\": null, \"scratchpad_state\": null, \"num\": 1, \"output\": \"eDP-1\", \"representation\": \"H[firefox]\", \"focused\": false, \"visible\": false }, { \"id\": 36, \"type\": \"workspace\", \"orientation\": \"horizontal\", \"percent\": null, \"urgent\": false, \"marks\": [ ], \"layout\": \"splith\", \"border\": \"none\", \"current_border_width\": 0, \"rect\": { \"x\": 0, \"y\": 0, \"width\": 1920, \"height\": 1080 }, \"deco_rect\": { \"x\": 0, \"y\": 0, \"width\": 0, \"height\": 0 }, \"window_rect\": { \"x\": 0, \"y\": 0, \"width\": 0, \"height\": 0 }, \"geometry\": { \"x\": 0, \"y\": 0, \"width\": 0, \"height\": 0 }, \"name\": \"2\", \"window\": null, \"nodes\": [ ], \"floating_nodes\": [ ], \"focus\": [ 39 ], \"fullscreen_mode\": 1, \"sticky\": false, \"floating\": null, \"scratchpad_state\": null, \"num\": 2, \"output\": \"eDP-1\", \"representation\": \"H[T[foot obsidian jetbrains-idea-ce]]\", \"focused\": true, \"visible\": true }, { \"id\": 19, \"type\": \"workspace\", \"orientation\": \"horizontal\", \"percent\": null, \"urgent\": false, \"marks\": [ ], \"layout\": \"splith\", \"border\": \"none\", \"current_border_width\": 0, \"rect\": { \"x\": 0, \"y\": 0, \"width\": 1920, \"height\": 1080 }, \"deco_rect\": { \"x\": 0, \"y\": 0, \"width\": 0, \"height\": 0 }, \"window_rect\": { \"x\": 0, \"y\": 0, \"width\": 0, \"height\": 0 }, \"geometry\": { \"x\": 0, \"y\": 0, \"width\": 0, \"height\": 0 }, \"name\": \"3\", \"window\": null, \"nodes\": [ ], \"floating_nodes\": [ ], \"focus\": [ 33 ], \"fullscreen_mode\": 1, \"sticky\": false, \"floating\": null, \"scratchpad_state\": null, \"num\": 3, \"output\": \"eDP-1\", \"representation\": \"H[T[thunderbird discord Spotify]]\", \"focused\": false, \"visible\": false }");
        let num: i32 = get_num_workspaces(&output);
        assert_eq!(num, 3);
    }
}
