//! Native check of menu transitions and screenshots at desktop and compact sizes.
use super::SmokeTest;
use crate::{
    app::*,
    scene::station::RETURN_POST,
    ui::{Menu, MenuAction, MenuPage},
};
use ainda_vivo_core::story::{Action, Phase};
use bevy::{app::AppExit, camera::visibility::RenderLayers, prelude::*, window::PrimaryWindow};

#[derive(Default)]
pub struct Route {
    frame: u32,
    key: Option<KeyCode>,
    button: Option<Entity>,
}

type Views<'w, 's> = (
    Query<'w, 's, &'static mut Window, With<PrimaryWindow>>,
    Query<'w, 's, &'static RenderLayers, With<MainCamera>>,
);

#[allow(
    clippy::too_many_arguments,
    reason = "Bevy injects native smoke test parameters."
)]
pub fn drive(
    mut smoke: ResMut<SmokeTest>,
    mut route: Local<Route>,
    menu: Res<Menu>,
    mut session: ResMut<Session>,
    (comfort, player): (Res<Comfort>, Res<LocalPlayer>),
    mut keys: ResMut<ButtonInput<KeyCode>>,
    (mut window, camera): Views,
    mut buttons: Query<(Entity, &MenuAction, &mut Interaction)>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
) {
    if !smoke.menu {
        return;
    }
    route.frame += 1;
    if let Some(key) = route.key.take() {
        keys.release(key);
    }
    if let Some(entity) = route.button.take()
        && let Ok((_, _, mut interaction)) = buttons.get_mut(entity)
    {
        *interaction = Interaction::None;
    }

    let key = match route.frame {
        26 => Some(KeyCode::ArrowDown),
        31 | 48 | 54 | 68 | 154 | 174 | 184 | 189 => Some(KeyCode::Enter),
        52 | 66 | 187 => Some(KeyCode::Tab),
        62 | 85 | 130 | 159 | 166 | 178 | 221 | 224 | 228 => Some(KeyCode::Escape),
        _ => None,
    };
    if let Some(key) = key {
        keys.press(key);
        route.key = Some(key);
    }
    let click = match route.frame {
        58 => Some(MenuAction::VolumeDown),
        120 => Some(MenuAction::Main),
        150 | 168 => Some(MenuAction::Wake),
        170 => Some(MenuAction::ConfirmWake),
        180 => Some(MenuAction::Settings),
        182 => Some(MenuAction::Accessibility),
        191 => Some(MenuAction::CrisisCamera),
        193 => Some(MenuAction::StabilityIndicator),
        _ => None,
    };
    if let Some(click) = click
        && let Some((entity, _, mut interaction)) =
            buttons.iter_mut().find(|(_, action, _)| **action == click)
    {
        *interaction = Interaction::Pressed;
        route.button = Some(entity);
    }

    let check = match route.frame {
        24 => Some((
            session.screen == Screen::Menu
                && menu.focus == MenuAction::Main
                && camera
                    .single()
                    .is_ok_and(|layers| *layers == RenderLayers::none()),
            "abertura_isolou_a_cena_3d_e_focou_jogar",
        )),
        28 => Some((
            !comfort.psycho_indicator && comfort.psycho_motion && comfort.psycho_av == 1.0,
            "acessibilidade_com_indicador_oculto_por_padrao",
        )),
        27 => Some((menu.focus == MenuAction::Settings, "seta_moveu_foco")),
        32 => Some((menu.page == MenuPage::Settings, "confirmou_configuracoes")),
        49 => Some((
            (comfort.fov_degrees - 60.0).abs() < 0.01,
            "diminiu_campo_de_visao",
        )),
        55 => Some((
            (comfort.fov_degrees - 65.0).abs() < 0.01,
            "campo_de_visao_restaurado",
        )),
        59 => Some((
            (comfort.ambience_volume - 0.55).abs() < 0.01,
            "botao_de_volume",
        )),
        63 => Some((
            session.screen == Screen::Menu
                && menu.page == MenuPage::Home
                && menu.focus == MenuAction::Settings,
            "escape_cedeu_o_foco_ao_botao_de_origem",
        )),
        69 => Some((menu.page == MenuPage::Controls, "painel_de_controles")),
        105 => Some((
            window
                .single()
                .is_ok_and(|window| window.width() <= 641.0 && window.height() <= 481.0),
            "janela_compacta",
        )),
        121 => Some((
            session.screen == Screen::Playing
                && session.attempt == 1
                && camera
                    .single()
                    .is_ok_and(|layers| *layers == RenderLayers::default()),
            "clique_iniciou_a_vistoria_e_restaurou_a_cena_3d",
        )),
        131 => Some((session.screen == Screen::Paused, "escape_pausou")),
        151 => Some((
            session.wake_confirmation && menu.focus == MenuAction::CancelWake,
            "despertar_focou_cancelar",
        )),
        155 => Some((
            session.screen == Screen::Paused && !session.wake_confirmation,
            "confirmacao_padrao_cancelou",
        )),
        160 => Some((session.screen == Screen::Playing, "escape_retomou")),
        172 => Some((
            session.screen == Screen::Complete && session.expedition.can_resume(),
            "confirmar_despertar_preservou_a_opcao_de_retomada",
        )),
        176 => Some((
            session.screen == Screen::Playing
                && session.progress().phase() == Phase::Return
                && player.body.feet.distance(RETURN_POST.feet) < 0.04,
            "enter_retomou_do_posto",
        )),
        183 => Some((
            session.screen == Screen::Paused
                && menu.page == MenuPage::Accessibility
                && menu.focus == MenuAction::PsychologicalEffectsDown,
            "abriu_acessibilidade_da_pausa",
        )),
        185 => Some(((comfort.psycho_av - 0.75).abs() < 0.01, "reduziu_efeitos")),
        195 => Some((
            comfort.psycho_indicator && !comfort.psycho_motion && comfort.psycho_av == 1.0,
            "restaurou_efeitos_e_alternou_camera_e_indicador",
        )),
        218 => Some((
            menu.page == MenuPage::Accessibility
                && window
                    .single()
                    .is_ok_and(|window| window.width() <= 641.0 && window.height() <= 481.0),
            "acessibilidade_na_janela_compacta",
        )),
        222 => Some((
            menu.page == MenuPage::Settings && menu.focus == MenuAction::Accessibility,
            "acessibilidade_retornou_as_configuracoes",
        )),
        225 => Some((
            menu.page == MenuPage::Home && session.screen == Screen::Paused,
            "voltar_das_configuracoes_preservou_pausa",
        )),
        229 => Some((
            session.screen == Screen::Playing,
            "retomada_apos_acessibilidade",
        )),
        _ => None,
    };
    if let Some((false, failure)) = check {
        smoke.failures.push(failure);
        error!("Menu: {failure}");
    }

    match route.frame {
        25 => smoke.capture(&mut commands, "01-tela-inicial"),
        45 => smoke.capture(&mut commands, "02-configuracoes"),
        81 => smoke.capture(&mut commands, "03-controles"),
        88 | 106 | 200 => {
            if let Ok(mut window) = window.single_mut() {
                let (width, height) = if route.frame == 88 || route.frame == 200 {
                    (640.0, 480.0)
                } else {
                    (1280.0, 720.0)
                };
                window.resolution.set(width, height);
            }
        }
        105 => smoke.capture(&mut commands, "04-tela-compacta"),
        148 => smoke.capture(&mut commands, "05-pausa"),
        163 => {
            for action in [Action::TakeFuse, Action::InspectPump, Action::RestorePower] {
                session.act(action);
            }
        }
        218 => smoke.capture(&mut commands, "06-acessibilidade-compacta"),
        245 => {
            if smoke.capture_dir.is_some() && smoke.captures != 6 {
                smoke.failures.push("capturas_incompletas");
            }
            let report = format!(
                "AINDA VIVO v{VERSION}\nsmoke-menu: {}\nchecks: focus and keyboard navigation, pointer actions, FOV and volume, controls screen, 1280x720 and 640x480, play, pause, neutral awakening confirmation, one post resume, compact accessibility, effects, camera and default-hidden indicator\ncaptures: {}\nfailures: {:?}\n",
                if smoke.failures.is_empty() {
                    "PASS"
                } else {
                    "FAIL"
                },
                smoke.captures,
                smoke.failures,
            );
            info!("{report}");
            let written = smoke.report.as_ref().is_none_or(|path| {
                std::fs::write(path, report)
                    .map_err(|error| error!("Falha no relatório: {error}"))
                    .is_ok()
            });
            exit.write(if written && smoke.failures.is_empty() {
                AppExit::Success
            } else {
                AppExit::error()
            });
        }
        _ => (),
    }
}
