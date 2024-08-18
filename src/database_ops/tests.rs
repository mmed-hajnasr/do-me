mod test_database_opearations {
    use pretty_assertions::assert_eq;

    use super::super::DatabaseOperations;
    use crate::structs::*;
    #[test]
    fn test_remove() {
        let database_connection = DatabaseOperations::new(":memory:".into());

        assert_eq!(database_connection.get_workspaces().unwrap(), vec![]);

        database_connection
            .handle_add_workspace(AddWorkspace {
                name: "Workspace 1".into(),
                ..Default::default()
            })
            .unwrap();

        let id = database_connection
            .search_workspace_name("Workspace 1")
            .unwrap()
            .unwrap();

        assert_eq!(database_connection.get_workspaces().unwrap().len(), 1);
        assert_eq!(database_connection.get_tasks(id).unwrap().len(), 0);

        database_connection
            .handle_add_task(AddTask {
                name: "Task 1".into(),
                workspace_id: id,
                ..Default::default()
            })
            .unwrap();

        database_connection
            .handle_add_task(AddTask {
                name: "Task 2".into(),
                workspace_id: id,
                ..Default::default()
            })
            .unwrap();

        let task1_id = database_connection
            .search_task_name("Task 1", id)
            .unwrap()
            .unwrap();
        let task2_id = database_connection
            .search_task_name("Task 2", id)
            .unwrap()
            .unwrap();

        assert_eq!(
            database_connection
                .get_task(task1_id)
                .unwrap()
                .unwrap()
                .name,
            "Task 1"
        );

        database_connection.handle_remove_task(task1_id).unwrap();

        assert_eq!(database_connection.get_task(task1_id).unwrap(), None);

        assert_eq!(database_connection.get_tasks(id).unwrap().len(), 1);

        database_connection.handle_remove_workspace(id).unwrap();

        assert_eq!(database_connection.get_workspaces().unwrap(), vec![]);

        assert_eq!(database_connection.get_tasks(id).unwrap(), vec![]);

        assert_eq!(database_connection.get_task(task2_id).unwrap(), None);
    }

    #[test]
    fn test_edit() {
        let database_connection = DatabaseOperations::new(":memory:".into());

        assert_eq!(database_connection.get_workspaces().unwrap(), vec![]);

        database_connection
            .handle_add_workspace(AddWorkspace {
                name: "Workspace 1".into(),
                ..Default::default()
            })
            .unwrap();

        let id = database_connection
            .search_workspace_name("Workspace 1")
            .unwrap()
            .unwrap();

        assert_eq!(database_connection.get_workspaces().unwrap().len(), 1);

        database_connection
            .handle_add_task(AddTask {
                name: "Task 1".into(),
                workspace_id: id,
                ..Default::default()
            })
            .unwrap();

        let task1_id = database_connection
            .search_task_name("Task 1", id)
            .unwrap()
            .unwrap();

        database_connection
            .handle_update_task(UpdateTask {
                id: task1_id,
                name: Some("Task 2".into()),
                description: Some("description".into()),
                ..Default::default()
            })
            .unwrap();

        let task = database_connection.get_task(task1_id).unwrap().unwrap();

        assert_eq!(task.name, "Task 2");

        assert_eq!(task.description.unwrap(), "description");

        assert_eq!(task.priority, 3);

        assert_eq!(task.completed, false);

        database_connection
            .handle_update_task(UpdateTask {
                id: task1_id,
                priority: Some(1),
                completed: Some(true),
                ..Default::default()
            })
            .unwrap();

        let task = database_connection.get_task(task1_id).unwrap().unwrap();

        assert_eq!(task.name, "Task 2");

        assert_eq!(task.description.unwrap(), "description");

        assert_eq!(task.priority, 1);

        assert_eq!(task.completed, true);
    }

    #[test]
    fn test_change_order_tasks() {
        let database_connection = DatabaseOperations::new(":memory:".into());
        let sorter = TaskSorter::Order(true);

        database_connection
            .handle_add_workspace(AddWorkspace {
                name: "Workspace 1".into(),
                ..Default::default()
            })
            .unwrap();

        let id = database_connection
            .search_workspace_name("Workspace 1")
            .unwrap()
            .unwrap();

        assert_eq!(database_connection.get_workspaces().unwrap().len(), 1);

        database_connection
            .handle_add_task(AddTask {
                name: "Task 1".into(),
                workspace_id: id,
                ..Default::default()
            })
            .unwrap();

        database_connection
            .handle_add_task(AddTask {
                name: "Task 2".into(),
                workspace_id: id,
                ..Default::default()
            })
            .unwrap();

        database_connection
            .handle_add_task(AddTask {
                name: "Task 3".into(),
                workspace_id: id,
                ..Default::default()
            })
            .unwrap();

        assert_eq!(
            database_connection
                .get_tasks(id)
                .unwrap()
                .into_iter()
                .map(|x| x.name)
                .collect::<Vec<String>>(),
            vec![
                "Task 1".to_string(),
                "Task 2".to_string(),
                "Task 3".to_string()
            ]
        );

        let task1_id = database_connection
            .search_task_name("Task 1", id)
            .unwrap()
            .unwrap();

        let task2_id = database_connection
            .search_task_name("Task 2", id)
            .unwrap()
            .unwrap();

        let task3_id = database_connection
            .search_task_name("Task 3", id)
            .unwrap()
            .unwrap();

        database_connection
            .handle_update_task(UpdateTask {
                id: task2_id,
                order: Some(0),
                ..Default::default()
            })
            .unwrap();

        let mut tasks = database_connection.get_tasks(id).unwrap();

        sorter.sort(&mut tasks);

        assert_eq!(
            tasks.into_iter().map(|x| x.name).collect::<Vec<String>>(),
            vec![
                "Task 2".to_string(),
                "Task 1".to_string(),
                "Task 3".to_string()
            ]
        );

        database_connection
            .handle_update_task(UpdateTask {
                id: task2_id,
                order: Some(2),
                ..Default::default()
            })
            .unwrap();

        let mut tasks = database_connection.get_tasks(id).unwrap();

        sorter.sort(&mut tasks);

        assert_eq!(
            tasks.into_iter().map(|x| x.name).collect::<Vec<String>>(),
            vec![
                "Task 1".to_string(),
                "Task 3".to_string(),
                "Task 2".to_string()
            ]
        );

        database_connection
            .handle_update_task(UpdateTask {
                id: task3_id,
                order: Some(0),
                ..Default::default()
            })
            .unwrap();

        let mut tasks = database_connection.get_tasks(id).unwrap();

        sorter.sort(&mut tasks);

        assert_eq!(
            tasks.into_iter().map(|x| x.name).collect::<Vec<String>>(),
            vec![
                "Task 3".to_string(),
                "Task 1".to_string(),
                "Task 2".to_string()
            ]
        );

        database_connection.handle_remove_task(task3_id).unwrap();

        let mut tasks = database_connection.get_tasks(id).unwrap();

        sorter.sort(&mut tasks);

        assert_eq!(
            tasks
                .into_iter()
                .map(|x| (x.order, x.name))
                .collect::<Vec<(i32, String)>>(),
            vec![(0, "Task 1".to_string()), (1, "Task 2".to_string())]
        );

        database_connection
            .handle_add_task(AddTask {
                name: "Task 3".into(),
                workspace_id: id,
                order: Some(1),
                ..Default::default()
            })
            .unwrap();

        let mut tasks = database_connection.get_tasks(id).unwrap();

        sorter.sort(&mut tasks);

        assert_eq!(
            tasks
                .into_iter()
                .map(|x| (x.order, x.name))
                .collect::<Vec<(i32, String)>>(),
            vec![
                (0, "Task 1".to_string()),
                (1, "Task 3".to_string()),
                (2, "Task 2".to_string())
            ]
        );
    }
    #[test]
    fn test_change_order_workspaces() {
        let database_connection = DatabaseOperations::new(":memory:".into());
        let sorter = WorkspaceSorter::Order(true);

        database_connection
            .handle_add_workspace(AddWorkspace {
                name: "Workspace 1".into(),
                ..Default::default()
            })
            .unwrap();

        database_connection
            .handle_add_workspace(AddWorkspace {
                name: "Workspace 2".into(),
                ..Default::default()
            })
            .unwrap();

        database_connection
            .handle_add_workspace(AddWorkspace {
                name: "Workspace 3".into(),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(
            database_connection
                .get_workspaces()
                .unwrap()
                .into_iter()
                .map(|x| x.name)
                .collect::<Vec<String>>(),
            vec![
                "Workspace 1".to_string(),
                "Workspace 2".to_string(),
                "Workspace 3".to_string()
            ]
        );

        let workspace1_id = database_connection
            .search_workspace_name("Workspace 1")
            .unwrap()
            .unwrap();

        let workspace2_id = database_connection
            .search_workspace_name("Workspace 2")
            .unwrap()
            .unwrap();

        let workspace3_id = database_connection
            .search_workspace_name("Workspace 3")
            .unwrap()
            .unwrap();

        database_connection
            .handle_update_workspace(UpdateWorkspace {
                id: workspace3_id,
                order: Some(0),
                ..Default::default()
            })
            .unwrap();

        let mut workspaces = database_connection.get_workspaces().unwrap();

        sorter.sort(&mut workspaces);

        assert_eq!(
            workspaces
                .into_iter()
                .map(|x| x.name)
                .collect::<Vec<String>>(),
            vec![
                "Workspace 3".to_string(),
                "Workspace 1".to_string(),
                "Workspace 2".to_string()
            ]
        );

        database_connection
            .handle_remove_workspace(workspace3_id)
            .unwrap();

        let mut workspaces = database_connection.get_workspaces().unwrap();

        sorter.sort(&mut workspaces);

        assert_eq!(
            workspaces
                .into_iter()
                .map(|x| (x.order, x.name))
                .collect::<Vec<(i32,String)>>(),
            vec![(0,"Workspace 1".to_string()),(1,"Workspace 2".to_string())]
        );

        database_connection
            .handle_add_workspace(AddWorkspace {
                name: "Workspace 3".into(),
                order: Some(1),
            })
            .unwrap();

        let mut workspaces = database_connection.get_workspaces().unwrap();

        sorter.sort(&mut workspaces);

        assert_eq!(
            workspaces
                .into_iter()
                .map(|x| (x.order, x.name))
                .collect::<Vec<(i32,String)>>(),
            vec![(0,"Workspace 1".to_string()),(1,"Workspace 3".to_string()),(2,"Workspace 2".to_string())]
        );
    }
}
