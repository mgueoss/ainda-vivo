# AINDA VIVO v0.0.12-dev.4 — A Caçada

Base funcional `60001c3` (dev.3). A perseguição usa o GLB e as animações da dev.2 e o piso, os portais e a física da dev.3. A representação física continua opcional via **F5** ou `--presence-physical`. [Handoff do Observador](AINDA-VIVO-HANDOFF-v0.0.12-dev.3.md) · [Dev Lab](docs/DEV-LAB.md) · [Validação](docs/VALIDACAO-v0.0.12-dev.4.md)

## Regras e responsáveis

| Local | Responsabilidade |
|---|---|
| `src/atmosphere/director.rs` | Intervalo, seleção da ameaça ou arrancada e resultado contextual do contato. |
| `src/lab/observer/hunt.rs` | Preparação, movimento, desistência, contato, freada e retomada. Tipos de sinais e de efeitos do contato. |
| `src/lab/observer/nav.rs` | Duas envoltórias da grade e rotas compartilhadas com o Observador. |
| `src/lab/observer.rs` | Progresso e segurança da campanha, sensores existentes e passagem dos resultados para a sessão. |
| `src/lab/observer/hunt_lab.rs` | Ajustes, corredor e sessão independente de ensaio. |
| `src/lab/observer/hunt_qa.rs` | Roteiro gráfico de corrida e captura Windows. |

O diretor só decide a partir de 56 s e da fase Elétrica, quando a cabeça da Presença já confirmou um alvo à distância, há espaço para preparar a ruptura e o operador tem saída. Instabilidade alta, textos, interação, pausa, refúgios e eventos protegidos cortam a oportunidade. Uma decisão autorizada coloca um intervalo de 115–163 s no diretor. A preparação varia entre 1,65 e 3 s, e 38% dessas ameaças são blefes.

A corrida padrão chega a 4,95 m/s com aceleração de até 4,2 m/s² e duração máxima de 7,5 s. As curvas limitam velocidade e guinada. A grade tem um perfil baixo próprio e a execução testa a projeção da envoltória orientada, incluindo a capacidade de levantar na saída. A memória de visão tem preferência. Ruídos recentes podem renovar uma investigação. A rota pode mudar após 0,34 s se o alvo percebido se deslocar ou se a porta mudar. A criatura perde o rastro em cerca de 2,15 s sem informação recente. A freada tem avanço limitado por colisão e distância do jogador.

O contato exige proximidade à frente e raio livre. Intimidação e desorientação pontuam uma vez no sistema psicológico existente. A desorientação pode reduzir a entrada de movimento em até 15% por 0,62 s respeitando conforto. A ilusão usa o VHS habilitado e dissipa a instância. Captura exige o posto real e chama `Expedition::capture` sem efeito especial. Despertar, progresso real e bloqueio de nova captura seguem o sistema original. O Lab usa outra `Session`.

`Signal` prepara ruptura, prontidão, aceleração, mãos, pés, respiração, vocalização, contato e retirada. A mixagem usa amostras existentes e o som da vocalização fica reservado para a etapa sonora. Os marcadores de mãos e pés vêm dos clipes e acompanham o ritmo de reprodução. A velocidade real ajusta o avanço das animações existentes. Curvas ainda dependem de mistura de clipes, sem reposicionamento procedural de cada apoio.

## Executar

```powershell
cargo build --locked --features dev-lab
./target/debug/ainda-vivo.exe --presence-physical --seed 20261118
./target/debug/ainda-vivo.exe --smoke-hunt --seed 20261118 --capture-dir capturas --smoke-report cacada.txt
```

O smoke posiciona os corpos e seleciona a captura em uma situação reproduzível. A aproximação, o contato, as consequências e a saída usam os sistemas do jogo. Preserve o teste de captura com a representação anterior e as rotas da campanha. A próxima etapa pode ligar a vocalização preparada e revisar o ritmo após testes humanos.
