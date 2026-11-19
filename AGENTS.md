# AINDA VIVO

## Direção permanente

- O usuário tem a direção criativa. Implemente em Rust com Bevy e preserve a física cinemática atual. A prioridade é Windows x64 e experiência solo. Planeje fronteiras para LAN sem assumir que há rede pronta.
- Use a Estação de Bombeamento 07 como um lugar de trabalho reconhecível. Deixe a rotina da Bomba 02 estabelecer a normalidade. Alterações pequenas, som direcional, silêncio e registros incompletos sustentam o terror.
- Dê ao jogador controle, pistas legíveis e espaço para recuar. Evite saltos de susto obrigatórios, perseguição contínua, flashes, gore e penalidades que obriguem a repetir a vistoria inteira.
- Preserve câmera, resposta da entrada, conforto e acessibilidade. Todo elemento visual importante deve continuar legível sem VHS. Respeite pausa, áudio mudo e reinício.
- Separe progresso autoritativo e percepção local. Um acontecimento visual não deve mudar colisão ou objetivos por acidente. Reutilize a geometria, mantenha o orçamento de renderização e exponha spoilers técnicos apenas em ferramentas explicitamente habilitadas.
- Dê contexto às ilusões com observações sustentadas, ações registradas e rotas. Preserve intervalos de normalidade. As cadeias são opcionais e a luz de trabalho pode oferecer uma pausa genuína.
- Despertar é abstrato e pede confirmação. Preserve o rearme e as leituras reais em uma retomada curta, respeitando os limites definidos no núcleo.
- Mantenha o mapa e as pistas originais. Use referências para decisões de ritmo, leitura do espaço e feedback, com crédito na documentação.

## Desenvolvimento e distribuição

- Leia CHANGELOG.md, docs/DISTRIBUICAO.md e o último AINDA-VIVO-HANDOFF antes de alterar os sistemas centrais.
- Reutilize dependências travadas em Cargo.lock e o cache local. Verifique formatação, Clippy e testes, e rode o smoke apropriado no Windows quando disponível. Diferencie medições, testes automatizados e avaliações humanas.
- Para mudanças narrativas e perceptivas, valide a rota da Bomba 02, o recuo e a captura silenciosa e a sequência opcional da OS 17. Os modos nativos são `--smoke-test`, `--smoke-reality` e `--smoke-echoes`.
- Preserve tags, ZIPs e releases anteriores. Use o inicializador universal e scripts/distribution para pacotes de jogador. Valide o executável e os SHA-256 antes de publicar. Publique somente quando autorizado e use um push regular.
- Documente mudanças de regra e limitações. Encerre o trabalho na versão solicitada.
