# A Presença na Estação 07

## v0.0.6 — A figura entre a luz e a escuridão

- A lanterna abre um feixe mais útil a curta distância. A escuridão aumenta a partir de 5,2 m e encobre as superfícies a 18,6 m, incluindo placas distantes. O VHS reduz a granulação nas sombras para preservar o contorno de objetos pouco iluminados. [Névoa por distância no Bevy](https://github.com/bevyengine/bevy/blob/v0.18.1/examples/3d/fog.rs)
- Seis pontos têm aparições. A primeira pode acontecer no caminho para a nave, com um fundo iluminado para revelar a forma. O vulto se desloca levemente ao surgir, tem um núcleo mais escuro com bordas transparentes e dura cerca de cinco segundos quando observado de uma distância segura. Se o jogador virar antes de a figura se formar no enquadramento, aquele ponto pode reaparecer depois.
- Ventoinhas giram atrás de grades e placas suspensas balançam em alguns setores. O quadro geral tem aros de metal, indicadores, parafusos e uma alavanca que muda de posição ao restaurar a energia. Bombas e terminais ganharam manômetros, flanges com parafusos e teclas individuais.

[Vulto com VHS suave](imagens/v0.0.6/triagem-vulto-vhs.png) · [Mesmo quadro depois que o vulto some](imagens/v0.0.6/triagem-sem-vulto.png) · [Nave escura ao fundo](imagens/v0.0.6/nave-alcance.png) · [Quadro geral](imagens/v0.0.6/quadro-energizado.png)

[Lanterna ligada](imagens/v0.0.6/lanterna-ligada.png) · [Lanterna desligada](imagens/v0.0.6/lanterna-desligada.png) · [Pás do exaustor no primeiro quadro](imagens/v0.0.6/exaustor-a.png) · [Pás do exaustor no segundo quadro](imagens/v0.0.6/exaustor-b.png)

## v0.0.5 — Primeira versão da criatura

Na v0.0.5, A Presença é uma silhueta preta e irregular. O fundo aparece através do corpo e as bordas se misturam à escuridão. A figura fica imóvel por alguns segundos e se desfaz mais cedo quando a câmera se fixa nela.

## Referências aplicadas

- O modo seguro de *SOMA* redesenhou o comportamento das criaturas para manter a atmosfera enquanto o jogador explora. Em AINDA VIVO, cada aparição ocupa um ponto fixo e a tensão vem da possibilidade de cruzar o olhar com a figura. [Frictional Games sobre o modo seguro de SOMA](https://frictionalgames.com/2017-11-what-is-somas-safe-mode/)
- Artur Kordas relacionou a distância da câmera e os detalhes pouco definidos de *Darkwood* ao espaço que o jogo deixa para a imaginação. A Presença usa um contorno irregular, transparência e uma forma apenas vagamente humana. [Entrevista com o cocriador de Darkwood](https://culture.pl/en/article/bringing-out-the-fear-in-players-imaginations-a-chat-with-artur-kordas-co-creator-of-video-game)
- No retrospecto de *Amnesia*, Thomas Grip descreveu jogadores reagindo a ruídos vagos que lembravam passos. A estação agora tem passos espaçados, batidas em canos, arrasto metálico e goteiras com pausas irregulares entre eles. [Frictional Games sobre nove anos fazendo jogos de terror](https://frictionalgames.com/2019-10-9-years-9-lessons-on-horror/)
- A equipe de *Dead Space* usou áreas pequenas de luz para dar peso aos trechos escuros entre elas. A Estação 07 usa a mesma ideia de contraste para deixar pisos, portas e máquinas dependerem do feixe da lanterna fora das áreas iluminadas. [Motive sobre iluminação e ritmo em Dead Space](https://www.ea.com/able/news/inside-dead-space-4-the-intensity-director)

## Comportamento na partida

- Cinco pontos têm aparições possíveis. Cada um exige linha de visão e começa a cerca de 14 a 28 m da câmera. Aproximar-se apaga a figura antes de chegar a 12 m. Cada ponto rende uma aparição por tentativa, com mais de 20 segundos de intervalo entre elas.
- A lanterna tem um feixe concentrado e uma borda suave. Origem, direção e sombras acompanham a câmera. As placas e luminárias dão referências entre os trechos em que o jogador precisa iluminar o chão.
- Os ruídos usam posições da estação, ganham direção conforme a câmera vira e ficam mais baixos e abafados atrás de paredes. O intervalo nominal é de 14 a 26 segundos. Quando um ruído está previsto durante uma aparição, ele aguarda alguns segundos depois que ela termina. O volume de ambiência controla todos os ruídos.
- A silhueta, as texturas e os sons são gerados localmente. A pausa congela as aparições e interrompe o áudio. Reiniciar abre uma nova tentativa.

## Capturas de validação

[Lanterna ligada](imagens/v0.0.5/lanterna-ligada.png) · [Lanterna desligada no mesmo ponto](imagens/v0.0.5/lanterna-desligada.png) · [Relatório da execução gráfica](smoke-report-v0.0.5.txt)

[Cena da nave antes da aparição](imagens/v0.0.5/nave-antes.png) · [Cena da nave durante a aparição](imagens/v0.0.5/nave-presenca.png)

As capturas usam uma câmera fixa para comparar a luz. A intensidade da sensação de estar sendo observado ainda depende de teste com pessoas jogando.
