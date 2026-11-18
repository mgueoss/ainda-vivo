# Ambientação da v0.0.3

O corredor é uma área de manutenção abandonada. A oficina soa familiar na ida e muda de caráter depois da retirada do fusível. A missão usa os mesmos espaços para que o jogador possa perceber o que mudou.

| Princípio | Aplicação no AINDA VIVO |
|---|---|
| Reconhecer um lugar antes de estranhá-lo | O zumbido elétrico tem origem na oficina. Ao voltar para o corredor, o som de fundo recua e dois passos discretos vêm da sala que ficou para trás. O sinal ocorre uma vez por tentativa. |
| Alternar tensão e alívio | Luz âmbar na entrada e na bancada, áreas frias no corredor e uma saída que se ilumina aos poucos depois que o quadro recebe energia. A luz da oficina baixa e volta suavemente no momento de silêncio. |
| Sugerir uma história pelo espaço | Uma ferramenta sobre a bancada, uma bandeja de fusível, folhas abandonadas e a ferrugem da ventilação dão pistas de trabalho interrompido. A sinalização e o fusível continuam legíveis. |
| Manter o som ligado ao lugar | Ventilação, eletricidade e passos têm posições próprias. Girar muda o equilíbrio estéreo. A distância atenua o som e as paredes abafam as frequências mais altas. |

As escolhas de expectativa e ambiguidade partem das observações dos criadores de *Amnesia*. A alternância de luz e silêncio segue o princípio de picos e vales descrito pela equipe da Motive. O uso de objetos e desgaste se baseia na apresentação de Smith e Worch sobre narrativa ambiental. [Frictional Games — 9 Years, 9 Lessons on Horror](https://frictionalgames.com/2019-10-9-years-9-lessons-on-horror/) · [Motive — The Intensity Director](https://www.ea.com/able/news/inside-dead-space-4-the-intensity-director) · [GDC — What Happened Here? Environmental Storytelling](https://www.gdcvault.com/play/1012647/What-Happened-Here-Environmental)

O estudo preliminar de Garner, Grimshaw e Abdel Nabi não demonstrou diferenças estatisticamente conclusivas nos parâmetros de volume, altura do som e panorâmica que testou. A mixagem aqui é uma hipótese de design para validar jogando. [Garner et al. — A Preliminary Experiment to Assess the Fear Value of Preselected Sound Parameters](https://vbn.aau.dk/ws/portalfiles/portal/61575828/a_preliminary_experiment_MG.pdf)

Sons, placas e texturas são gerados pelo próprio jogo. A reprodução usa o áudio do Bevy, com mixagem estéreo em função da orientação da câmera. O volume inicial é 65% e o menu permite ajustar ou silenciar. A pausa interrompe a reprodução e o reinício limpa o evento da tentativa anterior. [Documentação do áudio no Bevy 0.18.1](https://docs.rs/bevy/0.18.1/bevy/audio/index.html)

A versão para Windows passou em 18 testes e no percurso gráfico de 550 quadros na GTX 1660 Super. O teste conferiu a direção do estéreo em amostras, o início e a pausa da reprodução no Windows, a volta da oficina, a luz da saída e o reinício da missão. O HUD das capturas usa o tempo controlado desse teste. A reação de medo é o que fica para o teste com pessoas jogando. [Relatório de integração](smoke-report-v0.0.3.txt) · [Captura do corredor](imagens/v0.0.3/corredor.png)
