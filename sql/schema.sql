-- Pupose of this command: disable rucursive triggers because we don't want the delete trigger to activates the update trigger.
PRAGMA recursive_triggers = OFF;
-- Pupose of this command: activate foreign key constraints and delete on CASCADE.
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS Workspace (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  update_date datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  create_date datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  workspace_order INTEGER
);
CREATE TABLE IF NOT EXISTS Task (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  description TEXT,
  priority INTEGER NOT NULL DEFAULT 3,
  completed bit NOT NULL DEFAULT 0,
  create_date datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  task_order INTEGER,
  workspaceid INTEGER NOT NULL,
  FOREIGN KEY (workspaceid) REFERENCES Workspace(id) ON DELETE CASCADE
  UNIQUE(workspaceid, name)
);
 
CREATE TABLE IF NOT EXISTS trigger_control (active INTEGER);
INSERT INTO trigger_control VALUES (0);

CREATE TRIGGER IF NOT EXISTS updateWorkspaceUpdateDateOnTaskUpdate
AFTER
UPDATE ON Task BEGIN
UPDATE Workspace
SET update_date = CURRENT_TIMESTAMP
WHERE id = NEW.workspaceid;
END;

CREATE TRIGGER IF NOT EXISTS updateWorkspaceUpdateDateOnTaskInsert
AFTER
INSERT ON Task BEGIN
UPDATE Workspace
SET update_date = CURRENT_TIMESTAMP
WHERE id = NEW.workspaceid;
END;


CREATE TRIGGER IF NOT EXISTS updateTaskOrderOnTaskUpdate
BEFORE
UPDATE OF task_order ON Task 
WHEN (SELECT active FROM trigger_control) = 0
BEGIN
  UPDATE trigger_control SET active = 1;

  UPDATE Task
  SET task_order = task_order - 1
  WHERE NEW.task_order > OLD.task_order
  AND task_order > OLD.task_order
  AND task_order <= NEW.task_order
  AND workspaceid == NEW.workspaceid;

  UPDATE Task
  SET task_order = task_order + 1
  WHERE NEW.task_order < OLD.task_order
  AND task_order < OLD.task_order
  AND task_order >= NEW.task_order
  AND workspaceid == NEW.workspaceid;

  UPDATE trigger_control SET active = 0;
END;

CREATE TRIGGER IF NOT EXISTS updateTaskOrderOnTaskNullInsert
AFTER
INSERT ON Task
WHEN NEW.task_order IS NULL AND (SELECT active FROM trigger_control) = 0
BEGIN
  UPDATE trigger_control SET active = 1;
  UPDATE Task
  SET task_order = (SELECT COALESCE(MAX(task_order), -1) + 1 FROM Task WHERE workspaceid = NEW.workspaceid)
  WHERE id = NEW.id;
  UPDATE trigger_control SET active = 0;
END;

CREATE TRIGGER IF NOT EXISTS updateTaskOrderOnTaskNotNullInsert
BEFORE
INSERT ON Task
WHEN NEW.task_order IS NOT NULL AND (SELECT active FROM trigger_control) = 0
BEGIN
  UPDATE trigger_control SET active = 1;
  UPDATE Task
  SET task_order = task_order + 1
  WHERE task_order >= NEW.task_order
  AND workspaceid == NEW.workspaceid;
  UPDATE trigger_control SET active = 0;
END;

CREATE TRIGGER IF NOT EXISTS updateTaskOrderOnTaskDelete
BEFORE
DELETE ON Task 
WHEN (SELECT active FROM trigger_control) = 0
BEGIN
  UPDATE trigger_control SET active = 1;
  UPDATE Task
  SET task_order = task_order - 1
  WHERE task_order > OLD.task_order
  AND workspaceid == OLD.workspaceid;
  UPDATE trigger_control SET active = 0;
END;

CREATE TRIGGER IF NOT EXISTS updateWorkspaceOrderOnWorkspaceUpdate
BEFORE
UPDATE OF workspace_order ON Workspace
WHEN (SELECT active FROM trigger_control) = 0
BEGIN
  UPDATE trigger_control SET active = 1;

  UPDATE Workspace
  SET workspace_order = workspace_order - 1
  WHERE NEW.workspace_order > OLD.workspace_order
  AND workspace_order > OLD.workspace_order
  AND workspace_order <= NEW.workspace_order;

  UPDATE Workspace
  SET workspace_order = workspace_order + 1
  WHERE NEW.workspace_order < OLD.workspace_order
  AND workspace_order < OLD.workspace_order
  AND workspace_order >= NEW.workspace_order;
  UPDATE trigger_control SET active = 0;
END;

CREATE TRIGGER IF NOT EXISTS updateWorkspaceOrderOnWorkspaceNullInsert
AFTER
INSERT ON Workspace
WHEN NEW.workspace_order IS NULL AND (SELECT active FROM trigger_control) = 0
BEGIN
  UPDATE trigger_control SET active = 1;
  UPDATE Workspace
  SET workspace_order = (SELECT COALESCE(MAX(workspace_order), -1) + 1 FROM Workspace)
  WHERE id = NEW.id;
  UPDATE trigger_control SET active = 0;
END;

CREATE TRIGGER IF NOT EXISTS updateWorkspaceOrderOnWorkspaceNotNullInsert
BEFORE
INSERT ON Workspace
WHEN NEW.workspace_order IS NOT NULL AND (SELECT active FROM trigger_control) = 0
BEGIN
  UPDATE trigger_control SET active = 1;
  UPDATE Workspace
  SET workspace_order = workspace_order + 1
  WHERE workspace_order >= NEW.workspace_order;
  UPDATE trigger_control SET active = 0;
END;

CREATE TRIGGER IF NOT EXISTS updateWorkspaceOrderOnWorkspaceDelete
BEFORE
DELETE ON Workspace WHEN (SELECT active FROM trigger_control) = 0
BEGIN
  UPDATE trigger_control SET active = 1;
  UPDATE Workspace
  SET workspace_order = workspace_order - 1
  WHERE workspace_order > OLD.workspace_order;
  UPDATE trigger_control SET active = 0;
END;
