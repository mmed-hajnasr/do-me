mod test_database_opearations {

    use crate::database_ops::DatabaseOperations;
    use crate::structs::*;
    use pretty_assertions::assert_eq;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    const SEEDS: [u64; 5] = [432, 1323, 9923, 1425, 8239];
    fn process_workspaces(workspaces: &mut [Workspace]) -> Vec<(usize, String)> {
        let sorter = WorkspaceSorter::default();
        sorter.sort(workspaces);
        return workspaces
            .iter()
            .map(|workspace| (workspace.order, workspace.name.clone()))
            .collect();
    }

    fn process_tasks(tasks: &mut [Task]) -> Vec<(usize, String)> {
        let sorter = TaskSorter::default();
        sorter.sort(tasks);
        return tasks
            .iter()
            .map(|task| (task.order, task.name.clone()))
            .collect();
    }

    #[test]
    fn test_order_tasks() {
        for seed in SEEDS.iter() {
            let db = DatabaseOperations::new(":memory:".into());
            let mut target_tasks: Vec<String> = vec![];
            let mut rng = StdRng::seed_from_u64(*seed);

            db.handle_add_workspace(AddWorkspace {
                name: "the workspace".into(),
                ..Default::default()
            })
            .unwrap();
            let workspace_id = db.search_workspace_name("the workspace").unwrap().unwrap();

            let tasks = process_tasks(&mut db.get_tasks(workspace_id).unwrap());
            assert_eq!(tasks, vec![]);

            // adding 20 random tasks
            for _ in 0..20 {
                let task_name = rng.gen::<u32>().to_string();
                target_tasks.push(task_name.clone());
                db.handle_add_task(AddTask {
                    name: task_name.clone(),
                    workspace_id,
                    ..Default::default()
                })
                .unwrap();

                let tasks = process_tasks(&mut db.get_tasks(workspace_id).unwrap());
                assert_eq!(
                    tasks,
                    target_tasks
                        .iter()
                        .enumerate()
                        .map(|(i, name)| (i, name.clone()))
                        .collect::<Vec<(usize, String)>>()
                );
            }

            // inserting in the middle of the list
            for _ in 0..10 {
                let task_name = rng.gen::<u32>().to_string();
                let order = rng.gen_range(0..target_tasks.len());
                target_tasks.insert(order, task_name.clone());
                db.handle_add_task(AddTask {
                    name: task_name.clone(),
                    workspace_id,
                    order: Some(order),
                    ..Default::default()
                })
                .unwrap();

                let tasks = process_tasks(&mut db.get_tasks(workspace_id).unwrap());
                assert_eq!(
                    tasks,
                    target_tasks
                        .iter()
                        .enumerate()
                        .map(|(i, name)| (i, name.clone()))
                        .collect::<Vec<(usize, String)>>()
                );
            }

            // changing the order of the tasks
            for _ in 0..50 {
                let from = rng.gen_range(0..target_tasks.len());
                let to = rng.gen_range(0..target_tasks.len());
                let id = db
                    .search_task_name(&target_tasks[from], workspace_id)
                    .unwrap()
                    .unwrap();
                db.handle_update_task(UpdateTask {
                    id,
                    order: Some(to),
                    ..Default::default()
                })
                .unwrap();
                let temp = target_tasks[from].clone();
                target_tasks.remove(from);
                target_tasks.insert(to, temp);

                let tasks = process_tasks(&mut db.get_tasks(workspace_id).unwrap());
                assert_eq!(
                    tasks,
                    target_tasks
                        .iter()
                        .enumerate()
                        .map(|(i, name)| (i, name.clone()))
                        .collect::<Vec<(usize, String)>>()
                );
            }
        }
    }

    #[test]
    fn test_remove_tasks() {
        for seed in SEEDS.iter() {
            let db = DatabaseOperations::new(":memory:".into());
            let mut target_tasks: Vec<String> = vec![];
            let mut rng = StdRng::seed_from_u64(*seed);

            db.handle_add_workspace(AddWorkspace {
                name: "the workspace".into(),
                ..Default::default()
            })
            .unwrap();
            let workspace_id = db.search_workspace_name("the workspace").unwrap().unwrap();

            let tasks = process_tasks(&mut db.get_tasks(workspace_id).unwrap());
            assert_eq!(tasks, vec![]);

            // adding 20 random tasks
            for _ in 0..20 {
                let task_name = rng.gen::<u32>().to_string();
                target_tasks.push(task_name.clone());
                db.handle_add_task(AddTask {
                    name: task_name.clone(),
                    workspace_id,
                    ..Default::default()
                })
                .unwrap();

                let tasks = process_tasks(&mut db.get_tasks(workspace_id).unwrap());
                assert_eq!(
                    tasks,
                    target_tasks
                        .iter()
                        .enumerate()
                        .map(|(i, name)| (i, name.clone()))
                        .collect::<Vec<(usize, String)>>()
                );
            }

            // removing 10 tasks
            for _ in 0..10 {
                let to_remove = rng.gen_range(0..target_tasks.len());
                let id = db
                    .search_task_name(&target_tasks[to_remove], workspace_id)
                    .unwrap()
                    .unwrap();
                db.handle_remove_task(id).unwrap();
                target_tasks.remove(to_remove);

                let tasks = process_tasks(&mut db.get_tasks(workspace_id).unwrap());
                assert_eq!(
                    tasks,
                    target_tasks
                        .iter()
                        .enumerate()
                        .map(|(i, name)| (i, name.clone()))
                        .collect::<Vec<(usize, String)>>()
                );
            }

            // remove the workspace

            db.handle_remove_workspace(workspace_id).unwrap();
            let tasks = process_tasks(&mut db.get_tasks(workspace_id).unwrap());
            assert_eq!(tasks, vec![]);
        }
    }
    #[test]
    fn test_remove_workspaces() {
        for seed in SEEDS.iter() {
            let db = DatabaseOperations::new(":memory:".into());
            let mut target_workspaces: Vec<String> = vec![];
            let mut rng = StdRng::seed_from_u64(*seed);

            // adding 10 random workspaces
            for _ in 0..10 {
                let workspace_name = rng.gen::<u32>().to_string();
                target_workspaces.push(workspace_name.clone());
                db.handle_add_workspace(AddWorkspace {
                    name: workspace_name.clone(),
                    ..Default::default()
                })
                .unwrap();
            }

            let workspaces = process_workspaces(&mut db.get_workspaces().unwrap());
            assert_eq!(
                workspaces,
                target_workspaces
                    .iter()
                    .enumerate()
                    .map(|(i, name)| (i, name.clone()))
                    .collect::<Vec<(usize, String)>>()
            );

            // removing all workspaces
            for _ in 0..10 {
                let to_remove = rng.gen_range(0..target_workspaces.len());
                let id = db
                    .search_workspace_name(&target_workspaces[to_remove])
                    .unwrap()
                    .unwrap();
                db.handle_remove_workspace(id).unwrap();
                target_workspaces.remove(to_remove);

                let workspaces = process_workspaces(&mut db.get_workspaces().unwrap());
                assert_eq!(
                    workspaces,
                    target_workspaces
                        .iter()
                        .enumerate()
                        .map(|(i, name)| (i, name.clone()))
                        .collect::<Vec<(usize, String)>>()
                );
            }
        }
    }

    #[test]
    fn test_order_workspaces() {
        for seed in SEEDS.iter() {
            let db = DatabaseOperations::new(":memory:".into());
            let mut target_workspaces: Vec<String> = vec![];
            let mut rng = StdRng::seed_from_u64(*seed);

            // adding 10 random workspaces
            for _ in 0..10 {
                let workspace_name = rng.gen::<u32>().to_string();
                target_workspaces.push(workspace_name.clone());
                db.handle_add_workspace(AddWorkspace {
                    name: workspace_name.clone(),
                    ..Default::default()
                })
                .unwrap();
            }

            let workspaces = process_workspaces(&mut db.get_workspaces().unwrap());
            assert_eq!(
                workspaces,
                target_workspaces
                    .iter()
                    .enumerate()
                    .map(|(i, name)| (i, name.clone()))
                    .collect::<Vec<(usize, String)>>()
            );

            // inserting in the middle of the list
            for _ in 0..10 {
                let workspace_name = rng.gen::<u32>().to_string();
                let order = rng.gen_range(0..target_workspaces.len());
                target_workspaces.insert(order, workspace_name.clone());
                db.handle_add_workspace(AddWorkspace {
                    name: workspace_name.clone(),
                    order: Some(order),
                })
                .unwrap();

                let workspaces = process_workspaces(&mut db.get_workspaces().unwrap());
                assert_eq!(
                    workspaces,
                    target_workspaces
                        .iter()
                        .enumerate()
                        .map(|(i, name)| (i, name.clone()))
                        .collect::<Vec<(usize, String)>>()
                );
            }
            // changing the order of the workspaces

            for i in 0..50 {
                let from = rng.gen_range(0..target_workspaces.len());
                let to = rng.gen_range(0..target_workspaces.len());
                let id = db
                    .search_workspace_name(&target_workspaces[from])
                    .unwrap()
                    .unwrap();
                db.handle_update_workspace(UpdateWorkspace {
                    id,
                    order: Some(to),
                    ..Default::default()
                })
                .unwrap();
                let temp = target_workspaces[from].clone();
                target_workspaces.remove(from);
                target_workspaces.insert(to, temp);

                let workspaces = process_workspaces(&mut db.get_workspaces().unwrap());
                assert_eq!(
                    workspaces,
                    target_workspaces
                        .iter()
                        .enumerate()
                        .map(|(i, name)| (i, name.clone()))
                        .collect::<Vec<(usize, String)>>(),
                    "Failed at iteration: {} transfer {} to {}",
                    i,
                    from,
                    to
                );
            }
        }
    }
}
