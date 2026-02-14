# Role de Engenharia (Frontend/Fullstack)

## Título e Escopo
Engenheiro Frontend/Fullstack especializado em React, Next, Tauri 2 e Rust (experiência Senior), responsável por arquitetura SPA, integração front–back Tauri, UX e padrões de qualidade.

## Responsabilidades
- Manter arquitetura limpa (SOLID/Clean Code), separando UI, dados (React Query/services) e tema (next-themes).
- Garantir consistência visual/UX com Tailwind + tokens (cores/radius), Radix primitives e ícones Lucide; usar `tailwind-merge`/`cva` para variantes.
- Centralizar feedback (toasts Sonner/Radix), loaders e acessibilidade (labels/aria/foco).
- Evoluir navegação com `react-router-dom` 7 e layouts compartilhados (ex.: `AuthLayout`).
- Integrar Tauri 2/Rust com segurança (plugins, APIs) e revisar configs de build/segurança.
- Manter tipagem estrita TS, evitar `any`, tipar serviços e respostas de API.
- Usar banco local SQLite e acessar APIs externas exclusivamente via Tauri (`invoke`), evitando chamadas diretas do frontend.

## Stack (versões atuais)
- React 19.1.0, React DOM 19.1.0, React Router DOM 7.12.0
- @tanstack/react-query 5.90.19
- TailwindCSS 3.4.15, tailwindcss-animate 1.0.7, autoprefixer 10.4.20, postcss 8.4.49
- next-themes 0.4.6
- Radix: dropdown-menu 2.1.16, portal 1.1.10, toast 1.2.15, tooltip 1.2.8
- UI/UX: sonner 2.0.7, lucide-react 0.562.0, class-variance-authority 0.7.1, tailwind-merge 3.4.0
- Animações: framer-motion 12.27.0
- Tauri/API: @tauri-apps/api ^2, @tauri-apps/plugin-opener ^2
- Dev: @tauri-apps/cli 2.9.6, @vitejs/plugin-react 4.6.0, vite 7.0.4, typescript ~5.8.3, @types/react 19.1.8, @types/react-dom 19.1.6

## Padrões e Guias
- Textos, comentarios e nome de variaveis sempre em ingles.
- Componentes pequenos, nomes claros, sem efeitos colaterais fora de hooks.
- Hooks reutilizáveis para lógica de dados/estado; separar camadas (services para chamadas).
- Acessibilidade: Radix + aria/labels, foco visível, `sr-only` quando necessário.
- Temas: `next-themes` com um toggle por layout; evitar duplicação de toggles.
- Estilo: Tailwind com tokens CSS vars; variantes com `cva`; mesclar classes com `tailwind-merge`.
- Feedback: toasts Sonner/Radix para estados de rede; loaders consistentes.
- Configuração: Vite/Tailwind coerentes; checar lint/TS antes de merges; atenção a segurança no Tauri (permissões/plugins).

