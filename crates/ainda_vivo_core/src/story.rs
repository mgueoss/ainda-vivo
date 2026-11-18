/// Small prototype loop. Not the final campaign or creature AI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    TakeFuse,
    RestorePower,
    ReadNote,
}

#[derive(Clone, Debug, Default)]
pub struct Progress {
    pub has_fuse: bool,
    pub power_restored: bool,
    pub note_read: bool,
}

impl Progress {
    pub fn apply(&mut self, action: Action) -> &'static str {
        match action {
            Action::TakeFuse => {
                self.has_fuse = true;
                "Fusível recolhido. Passe pela nave de bombas e siga as placas para a sala elétrica."
            }
            Action::RestorePower if self.has_fuse => {
                self.power_restored = true;
                "Energia restaurada. A saída fica depois da decantação."
            }
            Action::RestorePower => "Falta um fusível. Procure na sala à esquerda.",
            Action::ReadNote => {
                self.note_read = true;
                "REGISTRO 01: Se o corredor parecer diferente, respire. Não siga a sombra."
            }
        }
    }

    pub fn objective(&self) -> &'static str {
        if self.power_restored {
            "Passe pela decantação e atravesse a saída iluminada."
        } else if self.has_fuse {
            "Leve o fusível ao quadro geral na sala elétrica."
        } else {
            "Encontre o fusível na sala à esquerda do corredor."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exit_requires_fuse_and_reset_restores_objective() {
        let mut p = Progress::default();
        p.apply(Action::RestorePower);
        assert!(!p.power_restored);
        p.apply(Action::TakeFuse);
        p.apply(Action::RestorePower);
        assert!(p.power_restored);
        p = Progress::default();
        assert!(!p.has_fuse && !p.power_restored && !p.note_read);
    }
}
