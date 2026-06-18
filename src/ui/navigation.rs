use crate::ui::state::{
    CommandPreviewAction, ConnectionSource, EngineOption, SoftDeleteStrategy, SqlPreviewAction,
};

pub(crate) fn next_connection_source(selected: ConnectionSource) -> ConnectionSource {
    match selected {
        ConnectionSource::Manual => ConnectionSource::ConfigFile,
        ConnectionSource::ConfigFile => ConnectionSource::Manual,
    }
}

pub(crate) fn previous_connection_source(selected: ConnectionSource) -> ConnectionSource {
    next_connection_source(selected)
}

pub(crate) fn next_engine_option(selected: EngineOption) -> EngineOption {
    match selected {
        EngineOption::PostgreSql => EngineOption::SqlServer,
        EngineOption::SqlServer => EngineOption::PostgreSql,
    }
}

pub(crate) fn previous_engine_option(selected: EngineOption) -> EngineOption {
    next_engine_option(selected)
}

pub(crate) fn next_soft_delete_strategy(selected: SoftDeleteStrategy) -> SoftDeleteStrategy {
    match selected {
        SoftDeleteStrategy::CreateDeletedAt => SoftDeleteStrategy::UseExistingField,
        SoftDeleteStrategy::UseExistingField => SoftDeleteStrategy::ContinueWithoutDelete,
        SoftDeleteStrategy::ContinueWithoutDelete => SoftDeleteStrategy::CreateDeletedAt,
    }
}

pub(crate) fn previous_soft_delete_strategy(selected: SoftDeleteStrategy) -> SoftDeleteStrategy {
    match selected {
        SoftDeleteStrategy::CreateDeletedAt => SoftDeleteStrategy::ContinueWithoutDelete,
        SoftDeleteStrategy::UseExistingField => SoftDeleteStrategy::CreateDeletedAt,
        SoftDeleteStrategy::ContinueWithoutDelete => SoftDeleteStrategy::UseExistingField,
    }
}

pub(crate) fn next_sql_preview_action(selected: SqlPreviewAction) -> SqlPreviewAction {
    match selected {
        SqlPreviewAction::ExecuteAndContinue => SqlPreviewAction::CopyAndContinue,
        SqlPreviewAction::CopyAndContinue => SqlPreviewAction::CopyAndStay,
        SqlPreviewAction::CopyAndStay => SqlPreviewAction::Cancel,
        SqlPreviewAction::Cancel => SqlPreviewAction::ExecuteAndContinue,
    }
}

pub(crate) fn previous_sql_preview_action(selected: SqlPreviewAction) -> SqlPreviewAction {
    match selected {
        SqlPreviewAction::ExecuteAndContinue => SqlPreviewAction::Cancel,
        SqlPreviewAction::CopyAndContinue => SqlPreviewAction::ExecuteAndContinue,
        SqlPreviewAction::CopyAndStay => SqlPreviewAction::CopyAndContinue,
        SqlPreviewAction::Cancel => SqlPreviewAction::CopyAndStay,
    }
}

pub(crate) fn next_command_preview_action(selected: CommandPreviewAction) -> CommandPreviewAction {
    match selected {
        CommandPreviewAction::ExecuteCommand => CommandPreviewAction::CopyAndMarkExternalExecution,
        CommandPreviewAction::CopyAndMarkExternalExecution => CommandPreviewAction::CopyAndStay,
        CommandPreviewAction::CopyAndStay => CommandPreviewAction::Cancel,
        CommandPreviewAction::Cancel => CommandPreviewAction::ExecuteCommand,
    }
}

pub(crate) fn previous_command_preview_action(
    selected: CommandPreviewAction,
) -> CommandPreviewAction {
    match selected {
        CommandPreviewAction::ExecuteCommand => CommandPreviewAction::Cancel,
        CommandPreviewAction::CopyAndMarkExternalExecution => CommandPreviewAction::ExecuteCommand,
        CommandPreviewAction::CopyAndStay => CommandPreviewAction::CopyAndMarkExternalExecution,
        CommandPreviewAction::Cancel => CommandPreviewAction::CopyAndStay,
    }
}

pub(crate) fn select_next(selected: &mut usize, total: usize) {
    if total == 0 {
        *selected = 0;
    } else {
        *selected = (*selected + 1) % total;
    }
}

pub(crate) fn select_previous(selected: &mut usize, total: usize) {
    if total == 0 {
        *selected = 0;
    } else if *selected == 0 {
        *selected = total - 1;
    } else {
        *selected -= 1;
    }
}
