import { useShallow } from "zustand/react/shallow";

import { useAppStore } from "./useAppStore";

export function useShellSlice() {
  return useAppStore(useShallow((state) => ({
    bootstrap: state.bootstrap,
    activeTab: state.activeTab,
    setActiveTab: state.setActiveTab,
    themeMode: state.themeMode,
    toggleTheme: state.toggleTheme,
    addTaskDialogOpen: state.addTaskDialogOpen,
    addTaskDialogContext: state.addTaskDialogContext,
    openAddTaskDialog: state.openAddTaskDialog,
    closeAddTaskDialog: state.closeAddTaskDialog,
    createTask: state.createTask,
    loading: state.loading,
    runtimeConfig: state.runtimeConfig,
    tagSchema: state.tagSchema,
    tagColorMap: state.tagColorMap,
    kanbanBoards: state.kanbanBoards
  })));
}

export function useSettingsSlice() {
  return useAppStore(useShallow((state) => ({
    settingsOpen: state.settingsOpen,
    openSettings: state.openSettings,
    closeSettings: state.closeSettings,
    dueConfig: state.dueNotificationConfig,
    duePermission: state.dueNotificationPermission,
    setDueNotificationsEnabled: state.setDueNotificationsEnabled,
    setDuePreNotifyEnabled: state.setDuePreNotifyEnabled,
    setDuePreNotifyMinutes: state.setDuePreNotifyMinutes,
    requestDueNotificationPermission: state.requestDueNotificationPermission,
    scanDueNotifications: state.scanDueNotifications
  })));
}

export function useDiagnosticsSlice() {
  return useAppStore(useShallow((state) => ({
    commandFailures: state.commandFailures,
    clearCommandFailures: state.clearCommandFailures
  })));
}

export function useTaskWorkspaceSlice() {
  return useAppStore(useShallow((state) => ({
    loading: state.loading,
    error: state.error,
    selectedTaskId: state.selectedTaskId,
    filters: state.taskFilters,
    setSearchFilter: state.setTaskSearchFilter,
    setStatusFilter: state.setTaskStatusFilter,
    setProjectFilter: state.setTaskProjectFilter,
    setTagFilter: state.setTaskTagFilter,
    setPriorityFilter: state.setTaskPriorityFilter,
    setDueFilter: state.setTaskDueFilter,
    clearFilters: state.clearTaskFilters,
    selectTask: state.selectTask,
    markTaskDone: state.markTaskDone,
    removeTask: state.removeTask
  })));
}

export function useKanbanWorkspaceSlice() {
  return useAppStore(useShallow((state) => ({
    error: state.error,
    loading: state.loading,
    openAddTaskDialog: state.openAddTaskDialog,
    boards: state.kanbanBoards,
    activeBoardId: state.activeKanbanBoardId,
    compactCards: state.kanbanCompactCards,
    draggingTaskId: state.draggingKanbanTaskId,
    dragOverLane: state.dragOverKanbanLane,
    filters: state.kanbanFilters,
    setActiveBoard: state.setActiveKanbanBoard,
    createBoard: state.createKanbanBoard,
    renameBoard: state.renameActiveKanbanBoard,
    deleteBoard: state.deleteActiveKanbanBoard,
    toggleCompact: state.toggleKanbanCompactCards,
    setDragging: state.setDraggingKanbanTask,
    setDragOver: state.setDragOverKanbanLane,
    moveTask: state.moveKanbanTask,
    moveTaskToBoard: state.moveKanbanTaskToBoard,
    markTaskDone: state.markTaskDone,
    removeTask: state.removeTask,
    setStatusFilter: state.setKanbanStatusFilter,
    setProjectFilter: state.setKanbanProjectFilter,
    setTagFilter: state.setKanbanTagFilter,
    setPriorityFilter: state.setKanbanPriorityFilter,
    setDueFilter: state.setKanbanDueFilter,
    clearFilters: state.clearKanbanFilters
  })));
}

export function useCalendarWorkspaceSlice() {
  return useAppStore(useShallow((state) => ({
    tasks: state.tasks,
    runtimeConfig: state.runtimeConfig,
    calendarView: state.calendarView,
    calendarFocusDateIso: state.calendarFocusDateIso,
    calendarTaskFilter: state.calendarTaskFilter,
    externalCalendars: state.externalCalendars,
    externalBusy: state.externalCalendarBusy,
    externalLastSync: state.externalCalendarLastSync,
    error: state.error,
    setCalendarView: state.setCalendarView,
    shiftCalendarFocus: state.shiftCalendarFocus,
    setCalendarTaskFilter: state.setCalendarTaskFilter,
    navigateCalendar: state.navigateCalendar,
    openNewExternalCalendar: state.openNewExternalCalendar,
    saveExternalCalendarSource: state.saveExternalCalendarSource,
    deleteExternalCalendarSource: state.deleteExternalCalendarSource,
    syncExternalCalendarSource: state.syncExternalCalendarSource,
    syncAllExternalCalendars: state.syncAllExternalCalendars,
    importExternalCalendarFile: state.importExternalCalendarFile
  })));
}
