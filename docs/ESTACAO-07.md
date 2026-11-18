# Estação de Bombeamento 07

A estação conecta 15 setores por 22 passagens, com uma área em planta de aproximadamente 6.100 m² num perímetro de 68 × 109 m. A oficina e a sala de registro fazem parte do setor de acesso. A área em planta inclui o espaço ocupado por paredes, mobiliário e água.

| Onde | O que diferencia o espaço |
|---|---|
| Acesso e triagem | Oficina original, registro, balcão e uma planta de orientação. |
| Carga e depósito | Portão de aço, engradados, paletes e estoque sob luz âmbar. |
| Arquivo e vestiário | Corredores entre fichários, armários, bancos e formulários abandonados. |
| Nave de bombas | Pé-direito de 6,2 m, motores de dois lados, volantes, adutoras e guias pintadas. |
| Filtros, tanques e inspeção | Reservatórios a alturas diferentes, dutos, vigas e rotas de ligação pelo lado oeste. |
| Geradores e sala elétrica | Ventilação, tubos de exaustão, módulos de distribuição e o quadro geral. |
| Controle, decantação e saída | Monitores de tubo, água rebaixada entre duas passarelas secas e um túnel iluminado após restaurar a energia. |

Os sons da oficina continuam no acesso. A nave tem um ruído mecânico baixo e a decantação tem goteiras. As placas físicas indicam os setores nos dois sentidos. A luz vai de âmbar perto das áreas de trabalho a fria nas zonas de água. O acionamento do quadro acende gradualmente as luminárias da saída.

O efeito VHS **Suave** é o padrão. **F4** alterna entre forte, desligado e suave. A preferência é lembrada entre execuções. São usadas granulação, linhas de rastreamento, vinheta discreta e separação de cores. A mira, os textos e a câmera mantêm posições estáveis. O passe roda depois do mapeamento de tons da cena 3D. [Fullscreen Material no Bevy 0.18](https://bevy.org/news/bevy-0-18/#fullscreen-material)

No teste em Windows com a GTX 1660 Super, a simulação atravessou o trajeto do acesso à sala elétrica e ao fim da decantação, abriu a comporta com o quadro e concluiu a expedição. A checagem de colisões percorreu as 22 passagens em ambos os sentidos e confirmou a saída isolada enquanto a comporta estava fechada. Os 18 testes automatizados passaram. A comparação dos pixels da nave confirmou que os três níveis produzem imagens distintas. [Relatório do percurso](smoke-report-v0.0.4.txt)

[Nave com VHS suave](imagens/v0.0.4/nave-vhs.png) · [Nave sem VHS](imagens/v0.0.4/nave-limpa.png) · [Nave com VHS forte](imagens/v0.0.4/nave-vhs-forte.png) · [Decantação](imagens/v0.0.4/decantacao.png) · [Arquivo](imagens/v0.0.4/arquivo.png) · [Carga](imagens/v0.0.4/carga.png)
