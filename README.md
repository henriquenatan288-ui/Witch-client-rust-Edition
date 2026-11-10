# Witch Client — Rust Edition

Client de mods para o **Horizon / Starling** (Minecraft reescrito em Rust), feito como *native trait mod*.

## Estado (honesto)
- `witchcore/`: o ClickGUI (abas, lista de modulos, ligar/desligar, scroll). **Testado**: `cd witchcore && cargo test` (7 testes passam).
- `engine/`: liga o ClickGUI aos hooks do Horizon (abre com **Right Shift**). **Nao compilado ainda** — o launcher gera o trait `Hooks` no build; os pontos incertos estao marcados com `CONFERIR`.
- A moldura do menu e uma imagem sua (`assets/frame.png`, 640x356, fundo transparente), enviada ao atlas do HUD com `mod_image` e desenhada com `sprite`. Se ela falhar, o menu cai para retangulos.
- Modulos com logica real (testada em `witchcore`): Player ESP, Tracers, Block ESP (grade de blocos), Fly, Trigger Bot. A parte que os liga ao jogo (`engine/`) nao foi compilada.
- Os demais da lista so guardam ligado/desligado.

## Instalar
`horizon-launcher native-build <esta pasta> --instance <instancia> --trust-native-code`
Depois abra o mundo e aperte Right Shift (no celular, ainda nao ha botao para abrir).

## Proximos passos
1. Compilar no launcher e corrigir o que o `CONFERIR` apontar.
2. Botao de toque para abrir o menu no Android.
3. Primeiros modulos reais: FPS, Coordinates, Radar/ESP (API `native_client_entities`).

## Estado honesto (atualizado)
- Testado: `witchcore` (24 testes, roda no CI).
- NAO compilado: `engine/` (precisa do launcher do Horizon; nunca foi testado no jogo/Starling).
- Trigger Bot: so detecta o alvo. A API do Horizon nao tem ataque do cliente e o pacote `Attack` exigiria `horizon-protocol`, entao nao ataca.
- Revisao do Horizon fixada em `1453e0e` (`horizon-mixin.toml`).
- Instalar: aba Mods > "Add from GitHub" com a URL deste repo, ou `horizon-launcher mods add-github <url> --trust-native-code`.
- Menu abre com Right Shift (ainda sem botao para celular).


## First test package

- The source-only package is produced by `python tools/package.py`.
- GitHub Actions also publishes `witch-0.1.0.hnmod` as the artifact `witch-client-first-test` after both Rust checks pass.
- The engine build still needs to be confirmed with the matching Horizon launcher and in a real 26.3 instance.
