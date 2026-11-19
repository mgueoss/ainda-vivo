/// The maintenance inspection and the few controls the operator can use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    TakeFuse,
    InspectPump,
    RestorePower,
    ReadNote,
    ReadArchive,
    ReadShiftLog,
    ReadReturnMemo,
    ReadBadge,
    ReviewRecording,
    CheckGate,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Phase {
    #[default]
    Fuse,
    Baseline,
    Electrical,
    Return,
    Exit,
}

#[derive(Clone, Debug, Default)]
pub struct Progress {
    phase: Phase,
    recording_reviewed: bool,
}

impl Progress {
    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn has_fuse(&self) -> bool {
        self.phase >= Phase::Baseline
    }

    pub fn power_restored(&self) -> bool {
        self.phase >= Phase::Return
    }

    pub fn exit_unlocked(&self) -> bool {
        self.phase == Phase::Exit
    }

    pub fn recording_reviewed(&self) -> bool {
        self.recording_reviewed
    }

    pub fn prompt(&self, action: Action) -> Option<&'static str> {
        Some(match action {
            Action::TakeFuse if self.has_fuse() => return None,
            Action::TakeFuse => "recolher fusível",
            Action::InspectPump if self.phase == Phase::Return => "registrar leitura de retorno",
            Action::InspectPump if self.phase >= Phase::Electrical => "conferir manômetro",
            Action::InspectPump => "registrar pressão na Bomba 02",
            Action::RestorePower if self.power_restored() => return None,
            Action::RestorePower => "rearmar quadro geral",
            Action::ReadNote => "ler ordem de vistoria",
            Action::ReadArchive => "ler ocorrência da Bomba 02",
            Action::ReadShiftLog => "ler livro de ponto",
            Action::ReadReturnMemo => "ler instrução do posto",
            Action::ReadBadge => "virar identificação",
            Action::ReviewRecording => "consultar gravação da nave",
            Action::CheckGate if self.exit_unlocked() => return None,
            Action::CheckGate => "conferir trava da comporta",
        })
    }

    pub fn apply(&mut self, action: Action) -> &'static str {
        match (action, self.phase) {
            (Action::TakeFuse, Phase::Fuse) => {
                self.phase = Phase::Baseline;
                "Fusível recolhido."
            }
            (Action::TakeFuse, _) => "O fusível já foi recolhido.",
            (Action::InspectPump, Phase::Fuse) => "A equipe deixou o fusível na oficina.",
            (Action::InspectPump, Phase::Baseline) => {
                self.phase = Phase::Electrical;
                "Pressão de repouso em zero. O lacre da última vistoria segue intacto."
            }
            (Action::InspectPump, Phase::Electrical) => "Pressão de repouso já registrada.",
            (Action::RestorePower, Phase::Fuse) => "Falta o fusível da oficina.",
            (Action::RestorePower, Phase::Baseline) => {
                "O quadro pede a leitura inicial da Bomba 02."
            }
            (Action::RestorePower, Phase::Electrical) => {
                self.phase = Phase::Return;
                "Rede rearmada. A trava aguarda a leitura de retorno."
            }
            (Action::RestorePower, _) => "O quadro já tem energia.",
            (Action::InspectPump, Phase::Return) => {
                self.phase = Phase::Exit;
                "Pressão acima da faixa. O mesmo lacre segue intacto. Comporta liberada."
            }
            (Action::InspectPump, Phase::Exit) => "A leitura de retorno ficou registrada.",
            (Action::ReadNote, _) => {
                "Ordem 07. A Bomba 02 está isolada desde o último turno. Compare as leituras antes e depois do rearme."
            }
            (Action::ReadArchive, _) => {
                "Arquivo, 03:12. Borges anotou três pancadas na Bomba 02. A equipe encontrou a prancheta no piso. A linha de saída dele ficou em branco."
            }
            (Action::ReadShiftLog, _) => {
                "Controle, 02:48. Borges deu baixa sozinho. Às 03:12, o gravador marcou dois operadores na nave. A folha original tem uma única assinatura."
            }
            (Action::ReadReturnMemo, _) => {
                "Posto da Elétrica. Se voltarem passos do mesmo vão, ilumine o acesso ou recue. Nos registros duplicados, preserve a última leitura que você anotou."
            }
            (Action::ReadBadge, _) => {
                "G. Borges, turno 07. No verso: «Falta só a leitura de retorno». A tinta ainda marca o plástico."
            }
            (Action::ReviewRecording, Phase::Fuse | Phase::Baseline | Phase::Electrical) => {
                "O gravador ainda depende do quadro geral."
            }
            (Action::ReviewRecording, _) => {
                self.recording_reviewed = true;
                "Câmera 02. O gravador salvou o quadro junto da leitura inicial. O campo de operador estava preenchido antes da vistoria."
            }
            (Action::CheckGate, Phase::Return) => "A trava pede a leitura de retorno na Bomba 02.",
            (Action::CheckGate, Phase::Exit) => "Comporta liberada.",
            (Action::CheckGate, _) => "A comporta aguarda o rearme e a vistoria da Bomba 02.",
        }
    }

    pub fn objective(&self) -> &'static str {
        match self.phase {
            Phase::Fuse => "Pegue o fusível na oficina à esquerda.",
            Phase::Baseline => "Vá à nave e registre o manômetro da Bomba 02.",
            Phase::Electrical => "Siga as setas âmbar até o quadro geral.",
            Phase::Return => "Volte à Bomba 02 para registrar a segunda leitura.",
            Phase::Exit => "Contorne a bacia de decantação e atravesse a saída.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inspection_requires_both_readings_and_rearm_in_order() {
        let mut p = Progress::default();
        p.apply(Action::InspectPump);
        p.apply(Action::RestorePower);
        assert_eq!(p.phase(), Phase::Fuse);
        p.apply(Action::TakeFuse);
        p.apply(Action::RestorePower);
        assert!(!p.power_restored());
        p.apply(Action::InspectPump);
        p.apply(Action::InspectPump);
        assert_eq!(p.phase(), Phase::Electrical);
        p.apply(Action::RestorePower);
        p.apply(Action::CheckGate);
        assert!(p.power_restored() && !p.exit_unlocked());
        p.apply(Action::InspectPump);
        p.apply(Action::TakeFuse);
        assert!(p.exit_unlocked());
        assert!(p.prompt(Action::CheckGate).is_none());
    }

    #[test]
    fn optional_recording_and_reading_do_not_advance_the_objective() {
        let mut p = Progress::default();
        p.apply(Action::ReadArchive);
        p.apply(Action::ReadShiftLog);
        p.apply(Action::ReadReturnMemo);
        p.apply(Action::ReadBadge);
        p.apply(Action::ReviewRecording);
        assert!(!p.recording_reviewed());
        p.apply(Action::TakeFuse);
        p.apply(Action::InspectPump);
        p.apply(Action::RestorePower);
        p.apply(Action::ReviewRecording);
        assert!(p.recording_reviewed());
        assert_eq!(p.phase(), Phase::Return);
        assert!(!Progress::default().recording_reviewed());
    }
}
