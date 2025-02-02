use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem;
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::menu::MenuEntry;
use crate::menu::{Menu, MenuSelectionResult};
use crate::profile::GameProfile;
use crate::shared_game_state::SharedGameState;

#[derive(Clone, Copy)]
pub struct MenuSaveInfo {
    pub current_map: u32,
    pub max_life: u16,
    pub life: u16,
    pub weapon_count: usize,
    pub weapon_id: [u32; 8],
}

impl Default for MenuSaveInfo {
    fn default() -> Self {
        MenuSaveInfo { current_map: 0, max_life: 0, life: 0, weapon_count: 0, weapon_id: [0; 8] }
    }
}
pub enum CurrentMenu {
    SaveMenu,
    DeleteConfirm,
}
pub struct SaveSelectMenu {
    pub saves: [MenuSaveInfo; 3],
    current_menu: CurrentMenu,
    save_menu: Menu,
    delete_confirm: Menu,
}

impl SaveSelectMenu {
    pub fn new() -> SaveSelectMenu {
        SaveSelectMenu {
            saves: [MenuSaveInfo::default(); 3],
            current_menu: CurrentMenu::SaveMenu,
            save_menu: Menu::new(0, 0, 200, 0),
            delete_confirm: Menu::new(0, 0, 75, 0),
        }
    }

    pub fn init(&mut self, state: &mut SharedGameState, ctx: &Context) -> GameResult {
        self.save_menu = Menu::new(0, 0, 200, 0);
        self.delete_confirm = Menu::new(0, 0, 75, 0);

        for (iter, save) in self.saves.iter_mut().enumerate() {
            if let Ok(data) = filesystem::user_open(ctx, state.get_save_filename(iter + 1).unwrap_or("".to_string())) {
                let loaded_save = GameProfile::load_from_save(data)?;

                save.current_map = loaded_save.current_map;
                save.max_life = loaded_save.max_life;
                save.life = loaded_save.life;
                save.weapon_count = loaded_save.weapon_data.iter().filter(|weapon| weapon.weapon_id != 0).count();
                save.weapon_id = loaded_save.weapon_data.map(|weapon| weapon.weapon_id);

                self.save_menu.push_entry(MenuEntry::SaveData(*save));
            } else {
                self.save_menu.push_entry(MenuEntry::NewSave);
            }
        }

        self.save_menu.push_entry(MenuEntry::Active("< Back".to_owned()));
        self.save_menu.push_entry(MenuEntry::Disabled("Press Right to Delete".to_owned()));

        self.delete_confirm.push_entry(MenuEntry::Disabled("Delete?".to_owned()));
        self.delete_confirm.push_entry(MenuEntry::Active("Yes".to_owned()));
        self.delete_confirm.push_entry(MenuEntry::Active("No".to_owned()));

        self.delete_confirm.selected = 2;

        self.update_sizes(state);

        Ok(())
    }

    fn update_sizes(&mut self, state: &SharedGameState) {
        self.save_menu.update_height();
        self.save_menu.x = ((state.canvas_size.0 - self.save_menu.width as f32) / 2.0).floor() as isize;
        self.save_menu.y = 30 + ((state.canvas_size.1 - self.save_menu.height as f32) / 2.0).floor() as isize;
        self.delete_confirm.update_height();
        self.delete_confirm.x = ((state.canvas_size.0 - self.delete_confirm.width as f32) / 2.0).floor() as isize;
        self.delete_confirm.y = 30 + ((state.canvas_size.1 - self.delete_confirm.height as f32) / 2.0).floor() as isize
    }

    pub fn tick(
        &mut self,
        exit_action: &mut dyn FnMut(),
        controller: &mut CombinedMenuController,
        state: &mut SharedGameState,
        ctx: &mut Context,
    ) -> GameResult {
        self.update_sizes(state);
        match self.current_menu {
            CurrentMenu::SaveMenu => match self.save_menu.tick(controller, state) {
                MenuSelectionResult::Selected(0, _) => {
                    state.save_slot = 1;
                    state.reload_resources(ctx)?;
                    state.load_or_start_game(ctx)?;
                }
                MenuSelectionResult::Selected(1, _) => {
                    state.save_slot = 2;
                    state.reload_resources(ctx)?;
                    state.load_or_start_game(ctx)?;
                }
                MenuSelectionResult::Selected(2, _) => {
                    state.save_slot = 3;
                    state.reload_resources(ctx)?;
                    state.load_or_start_game(ctx)?;
                }
                MenuSelectionResult::Selected(3, _) | MenuSelectionResult::Canceled => exit_action(),
                MenuSelectionResult::Right(slot, _, _) => {
                    if slot <= 2 {
                        self.current_menu = CurrentMenu::DeleteConfirm;
                        self.delete_confirm.selected = 2;
                    }
                }
                _ => (),
            },
            CurrentMenu::DeleteConfirm => match self.delete_confirm.tick(controller, state) {
                MenuSelectionResult::Selected(1, _) => {
                    state.sound_manager.play_sfx(17); // Player Death sfx
                    filesystem::user_delete(
                        ctx,
                        state.get_save_filename(self.save_menu.selected + 1).unwrap_or("".to_string()),
                    )?;
                    self.save_menu.entries[self.save_menu.selected] = MenuEntry::NewSave;
                    self.current_menu = CurrentMenu::SaveMenu;
                }
                MenuSelectionResult::Selected(2, _) | MenuSelectionResult::Canceled => {
                    self.current_menu = CurrentMenu::SaveMenu;
                }
                _ => (),
            },
        }

        Ok(())
    }

    pub fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        match self.current_menu {
            CurrentMenu::SaveMenu => {
                self.save_menu.draw(state, ctx)?;
            }
            CurrentMenu::DeleteConfirm => {
                self.delete_confirm.draw(state, ctx)?;
            }
        }
        Ok(())
    }
}
