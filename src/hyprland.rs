use hyprland::data::Client;
use hyprland::data::Monitors;
use hyprland::data::Workspace;
use hyprland::data::Workspaces;
use hyprland::dispatch::{Dispatch, DispatchType, WorkspaceIdentifierWithSpecial};
use hyprland::prelude::*;
use hyprnome::WorkspaceState;
use std::collections::HashMap;

pub fn get_state() -> hyprland::Result<WorkspaceState> {
    fn generate_next_level(prev_level: &Vec<i32>) -> Vec<i32> {
        let mut next_level = Vec::new();

        // Add the midpoint before the first element
        next_level.push(prev_level[0] / 2);

        // Add the midpoints between each pair of elements
        for window in prev_level.windows(2) {
            let midpoint = ((window[0] as i64 + window[1] as i64) / 2) as i32;
            next_level.push(midpoint);
        }

        // Add the midpoint after the last element
        let last_element = *prev_level.last().unwrap() as i64;
        next_level.push((last_element + (last_element / 2)) as i32);

        next_level
    }
    let monitors = Monitors::get()?;
    let workspaces = Workspaces::get()?.filter(|workspace| workspace.id > 0);
    let current_id = Workspace::get_active()?.id;

    let monitor_ids: Vec<i32> = workspaces
        .clone()
        .filter(|workspace| {
            if let Some(monitor) = monitors.clone().find(|monitor| monitor.focused) {
                workspace.monitor == monitor.name
            } else {
                false
            }
        })
        .map(|workspace| workspace.id)
        .collect();

    let occupied_ids: Vec<i32> = workspaces.clone().map(|workspace| workspace.id).collect();

    // Create a HashMap to hold the workspace ID and the number of windows
    let mut workspace_windows: HashMap<i32, u16> = HashMap::new();

    // Populate the HashMap
    for workspace in workspaces {
        workspace_windows.insert(workspace.id, workspace.windows);
    }
    // Initialize the first level
    let mut level: Vec<i32> = (1..=30).map(|x| (2u64.pow(31) / 31 * x as u64) as i32).collect();
    // let mut level: Vec<u64> = (1..=15).map(|x| 2u64.pow(32) / 16 * x).collect();

    // Calculate the number of levels
    let smallest_diff = level[1] - level[0];
    let levels = (smallest_diff as f64).log2() as i32;

    // Initialize 2D graph_matrix with the first level
    let mut graph_matrix: Vec<Vec<i32>> = Vec::new();
    graph_matrix.push(level.clone());

    // Generate and print the next levels
    for i in 2..=levels {
        level = generate_next_level(&level);
        graph_matrix.push(level.clone());
    }

    Ok(WorkspaceState::new(current_id, monitor_ids, occupied_ids, workspace_windows, graph_matrix))
}

/// Gets whether the current workspace is a special workspace or not.
///
/// This function works by getting which workspace the active window is in.
///
/// The if statement is used to make sure this function works when no window
/// is the active window.
pub fn is_special() -> hyprland::Result<bool> {
    if let Some(client) = Client::get_active()? {
        let Client { workspace, .. } = client;

        return Ok(workspace.name.contains("special"));
    }

    Ok(false)
}

pub fn change_workspace(id: i32, _move: bool, keep_special: bool) -> hyprland::Result<()> {
    let id = WorkspaceIdentifierWithSpecial::Id(id);

    if _move {
        let was_special = is_special()?;

        hyprland::dispatch!(MoveToWorkspace, id, None)?;

        if !keep_special && was_special {
            hyprland::dispatch!(ToggleSpecialWorkspace, None)
        } else {
            Ok(())
        }
    } else {
        if !keep_special && is_special()? {
            hyprland::dispatch!(ToggleSpecialWorkspace, None)?;
        }

        hyprland::dispatch!(Workspace, id)
    }
}
