use crate::executors::robot::{ColumnsMode, RowsMode};
use crate::kumir_state::{KumirState, Modes};
use egui::{Response, Ui, Widget};

pub struct RobotWidget<'a> {
    pub kumir_state: &'a mut KumirState,
}

impl Widget for RobotWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        match self.kumir_state.selected_mode.clone() {
            Modes::Robot(rob) => {
                let mut rob = rob.lock().unwrap();
                ui.scope(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Добавление/удаление столбцов: ");
                        ui.selectable_value(
                            &mut rob.editing_state.deleting_columns_mode,
                            ColumnsMode::FromRight,
                            "Справа",
                        );
                        ui.selectable_value(
                            &mut rob.editing_state.deleting_columns_mode,
                            ColumnsMode::FromLeft,
                            "Слева",
                        );
                    });
                    ui.end_row();
                    ui.horizontal(|ui| {
                        if ui.button("-").clicked() {
                            match rob.editing_state.deleting_columns_mode {
                                ColumnsMode::FromLeft => rob.remove_column_from_left(),
                                ColumnsMode::FromRight => rob.remove_column_from_right(),
                            }
                        }
                        ui.label(format!("{}", rob.get_width()));
                        if ui.button("+").clicked() {
                            match rob.editing_state.deleting_columns_mode {
                                ColumnsMode::FromLeft => rob.add_column_from_left(),
                                ColumnsMode::FromRight => rob.add_column_from_right(),
                            }
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Добавление/удаление строк: ");
                        ui.selectable_value(
                            &mut rob.editing_state.deleting_rows_mode,
                            RowsMode::FromDown,
                            "Снизу",
                        );
                        ui.selectable_value(
                            &mut rob.editing_state.deleting_rows_mode,
                            RowsMode::FromUp,
                            "Сверху",
                        );
                    });
                    ui.end_row();
                    ui.horizontal(|ui| {
                        if ui.button("-").clicked() {
                            match rob.editing_state.deleting_rows_mode {
                                RowsMode::FromDown => rob.remove_row_from_down(),
                                RowsMode::FromUp => rob.remove_row_from_up(),
                            }
                        }
                        ui.label(format!("{}", rob.get_height()));
                        if ui.button("+").clicked() {
                            match rob.editing_state.deleting_rows_mode {
                                RowsMode::FromDown => rob.add_row_from_down(),
                                RowsMode::FromUp => rob.add_row_from_up(),
                            }
                        }
                    });
                })
                .response
            }
            _ => ui.response(),
        }
    }
}
