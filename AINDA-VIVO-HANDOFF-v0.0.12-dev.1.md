# AINDA VIVO v0.0.12-dev.1

Etapa 1 da v0.0.12. A base preservada é o commit `6afe2a5`, v0.0.11 — Colapso. A branch local `dev/v0.0.12-dev.1` contém o estudo 3D original da Presença e seu laboratório. [Handoff da v0.0.11](AINDA-VIVO-HANDOFF-v0.0.11.md) · [Atalhos e uso do Dev Lab](docs/DEV-LAB.md) · [Validação e capturas](docs/VALIDACAO-v0.0.12-dev.1.md)

## Recurso 3D

- Fonte editável em `art/presenca/presenca.blend`. Três PNGs PBR também estão em `art/presenca/texturas` e empacotados no Blender.
- Recurso de execução em `assets/models/presenca.glb`. A escala usa metros. A frente é `−Y` no Blender e `+Z` no Bevy. O ponto de inserção em pé está no chão.
- `scripts/presenca/anatomia.py` define volumes loftados e modulações para torso, escápulas, vértebras, face e dedos. O Blender une os volumes em uma superfície fechada, suaviza e decima. Os pesos são transferidos das seções anatômicas e relaxados ao longo da conectividade real da malha. São no máximo quatro influências normalizadas por vértice.
- `scripts/presenca/superficie.py` cria UVs e pinta cor, rugosidade e relevo normal com base na posição anatômica. Cor e normal têm 1024² pixels. Rugosidade tem 512². Há um material de superfície com variação de tecido, pele e vestígio de colar.
- `scripts/presenca/poses.py` cria 52 ossos, pesos de deformação linear, poses e um aceno curto de inspeção. As ações guardadas no Blender exportam como clipes no GLB.

No Blender, selecione **Presença · rig**. Use o Editor de Ações para escolher um dos clipes salvos. Na edição não linear, as pistas estão silenciadas para abrir o modelo na pose neutra. Escolha uma pista por vez para inspecionar sua deformação. A visualização de material ou o Dev Lab mostram os mapas PBR.

### Regerar

```powershell
blender --background --factory-startup --python-exit-code 1 --python scripts/presenca/gerar.py
```

Foi usado Blender 4.5.16 LTS Windows x64. Para preparar uma revisão em outra pasta, acrescente `-- CAMINHO_DE_SAIDA`. A saída registra topologia, pesos e medidas das poses. [Blender 4.5 LTS](https://www.blender.org/releases/4-5/)

### Montar o pacote experimental

```powershell
cargo build --locked --features dev-lab
./scripts/distribution/notices.ps1 -OutputFile "$PWD/LICENCAS-dev.txt" -Features dev-lab
./scripts/presenca/empacotar-dev.ps1 -GameExe CAMINHO_DO_EXE -NoticesFile "$PWD/LICENCAS-dev.txt" -OutputDirectory CAMINHO_DE_SAIDA
```

O empacotador cria um ZIP separado com SHA-256, o executável, as licenças e o comando de abertura do Lab. O catálogo oficial de distribuição continua apontando para os pacotes já publicados.

## Fronteiras de código

| Camada | Responsabilidade |
|---|---|
| `src/app.rs` | A agenda da campanha usa a condição de execução do Lab no passo fixo e nos sistemas de entrada e atualização. |
| `src/lab.rs` | Ações, ativação de câmeras, suspensão e retomada dos sinks da campanha, configuração das fontes e ciclo de vida. |
| `src/lab/settings.rs` | Estado serializável independente, presets, filtros, instâncias e validação de faixas. |
| `src/lab/library.rs` | Catálogo, GLB/skin, clipes Bevy, materiais de inspeção, dados reais da malha, comparação e prévia. |
| `src/lab/room.rs` | Sala na camada própria de render, física do operador, câmera, luzes, névoa e VHS. |
| `src/lab/panel.rs` | Interface, busca, biblioteca, inspeção e ajustes isolados do menu do jogo. |
| `src/lab/qa.rs` | Roteiro nativo de teste e de captura PNG. |

O Dev Lab usa o `PlayerBody` da campanha com outra lista de sólidos e outra câmera. A sala tem estado próprio de posição, entrada, objetos e preferências. Ao fechar, as instâncias experimentais são removidas. A sala, sua câmera, luzes e interface ficam inativas para reuso no próximo acesso. O recurso GLB e a biblioteca também são reutilizados. As câmeras e os áudios previamente ativos da campanha recuperam seu estado.

O esqueleto usa as posições globais dos joints importados. O wireframe reconstrói as arestas da malha visível para inspeção e aplica os mesmos pesos e matrizes de ligação em CPU, com reuso enquanto a pose fica parada. A representação anterior usa um recorte plano gerado da máscara usada pela campanha.

## Próxima etapa sujeita à direção

Usar as vistas frontal, lateral, traseira, rosto, mãos e apoio para aprovar a linguagem visual. Antes de integrar animações de locomoção, validar um ciclo com contatos de mãos e pés, deslocamento do tronco e limites das juntas. Os clipes atuais servem para inspecionar articulações. A campanha mantém o modelo antigo até essa decisão.
