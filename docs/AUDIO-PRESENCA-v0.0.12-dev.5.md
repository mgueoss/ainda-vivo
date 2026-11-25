# Áudio da Presença · v0.0.12-dev.5

Toda a biblioteca é original e sintetizada por código em Rust. A build funciona offline e gera o banco uma vez, sem capturas de voz, amostras externas, serviços ou dependências adicionais. O código em `src/atmosphere/sound/presence_bank.rs` é a fonte dos recursos.

| Grupos | Famílias | Construção |
|---|---|---|
| Humanoide | Passada lenta, passada acelerada, aproximação | Impacto modal grave, atrito curto, camada orgânica e ataque variável |
| Quadrúpede | Apoio de mão, apoio de pé, arrancada, freada, parada brusca | Ataques e durações distintos, ruído filtrado e ressonância |
| Vocal | Respiração irregular, garganta/ruptura, gemido distante, grito deformado, vocalização aguda | Excitação glotal sintética, formantes móveis, irregularidade de frequência, ruído de ar e envoltórias |

Cada família tem três variações, três superfícies e versões seca e refletida, com amostras mono a 24 kHz. O filtro de superfícies usa o setor da fonte para selecionar metal, concreto ou área molhada. O banco mantém 117 entradas, duas formas de onda por entrada. A cauda refletida usa três atrasos fixos por superfície, com ganho limitado e filtragem. A demonstração WAV opcional usa o decodificador estéreo do jogo e uma sequência de audição fixa.

## Fluxo em execução

1. `lab/library.rs` lê os marcadores dos clipes humanoides 13/14/19 e quadrúpedes 26–28. A origem usa a transformação do ator e a lateral do membro. As posições dos apoios são aproximadas em relação ao corpo. As fases de caçada sinalizam ruptura, preparação, aceleração, respiração, vocalização, frenagem, parada e retirada pelo mesmo canal.
2. `presence_audio.rs` classifica os pedidos como manifestação física, ilusão do diretor ou falsa realidade. A categoria é visível apenas no laboratório. Todas passam pelo mesmo banco, filtros e espacialização. Pedidos de som são independentes dos estímulos lógicos de audição da IA.
3. O misturador seleciona a dupla de canais de acordo com posição, distância e eixo direito da câmera. Atenua a fonte por obstruções e conexão entre setores. A comporta entra e sai da geometria acústica junto à condição existente da missão. Cada fonte atualiza a obstrução a cada 95 ms. O decodificador interpola ganhos e filtro.
4. A mixagem usa alcance máximo de 36 m, até sete fontes da Presença e duas vocalizações simultâneas. O ganho por fonte diminui conforme o número de vozes ativas. Fontes já em execução são silenciadas ao pausar ou mudar de sessão, e o laboratório limpa suas fontes ao sair.
5. A camada respeita o volume de ambiência, o mudo global e a intensidade audiovisual existentes. A regulagem de volume e pitch da aba **Som** é local ao ensaio. Alterar o pitch muda a taxa de reprodução e a duração da amostra.

## Direção das ilusões

A primeira janela própria pode começar após aproximadamente 39 segundos da entrada na tentativa ou de sua revisão perceptiva. As seguintes aguardam 92–150 segundos e uma janela de som do diretor. A origem vem do lado vizinho de um portal validado com os colisores existentes. A fonte fica fora do foco imediato ou atrás de geometria e percorre um segmento curto naquele setor.

O catálogo inclui impacto seguido de silêncio, respiração que para sob o olhar, passos que suspendem a cadência quando o jogador para, voz distante e caçada invisível. Na caçada invisível há aceleração de cadência, alternância de mãos e pés, vocalização eventual e variantes de recuo ou interrupção. A ilusão termina se ficar perto demais do ouvinte. Ao final, uma pequena chance de solicitar uma manifestação futura usa as regras normais do diretor. As pistas sonoras existentes da missão têm prioridade no mesmo quadro.

A rotina cede a manifestações físicas, leitura, descanso, refúgio, episódios protegidos e estados de alta instabilidade. Um impacto ou caçada próxima e audível pode gerar um único estímulo psicológico de baixa intensidade. Os pulsos posteriores são apenas áudio. A captura silenciosa e o Despertar mantêm seus contratos autoritativos.

## Limites

O áudio usa estéreo panorâmico, conexão por portais e um filtro simples. A reverberação é uma assinatura curta da superfície. A trajetória invisível é um segmento no setor vizinho. A mistura e o caráter vocal ainda precisam de avaliação humana em fones e caixas. [Validação técnica](VALIDACAO-v0.0.12-dev.5.md)

## Demonstração estéreo

O teste abaixo grava seis segundos com o decodificador do jogo.

```powershell
$env:AV_PRESENCE_DEMO = Join-Path $PWD audicao-presenca.wav
cargo test --profile dev --locked --offline -p ainda-vivo --features dev-lab audition_pcm_uses_game_decoder
Remove-Item Env:AV_PRESENCE_DEMO
```
