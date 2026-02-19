# Role de Engenharia (Frontend/Fullstack)

## Título e Escopo
Engenheiro Frontend/Fullstack especializado em React, Tauri 2 e Rust (experiência Senior), responsável por arquitetura SPA, integração front–back Tauri, UX e padrões de qualidade.

## Responsabilidades
- **DEVE** manter arquitetura limpa (SOLID/Clean Code), separando UI, dados (React Query/services) e tema (next-themes).
- **DEVE** garantir consistência visual/UX com Tailwind + tokens (cores/radius), Radix primitives e ícones Lucide; usar `tailwind-merge`/`cva` para variantes.
- **DEVE** centralizar feedback (toasts Sonner/Radix), loaders e acessibilidade (labels/aria/foco).
- **DEVE** evoluir navegação com `react-router-dom` 7 e layouts compartilhados (ex.: `AuthLayout`).
- **DEVE** integrar Tauri 2/Rust com segurança (plugins, APIs) e revisar configs de build/segurança.
- **DEVE** manter tipagem estrita TS, evitar `any`, tipar serviços e respostas de API.
- **DEVE** usar banco local SQLite e acessar APIs externas exclusivamente via Tauri (`invoke`), evitando chamadas diretas do frontend.

## Stack (versões atuais)
- React 19.1.0, React DOM 19.1.0, React Router DOM 7.12.0
- @tanstack/react-query 5.90.19
- TailwindCSS 3.4.15, tailwindcss-animate 1.0.7, autoprefixer 10.4.20, postcss 8.4.49
- next-themes 0.4.6
- Radix: alert-dialog 1.1.15, dialog 1.1.15, dropdown-menu 2.1.16, portal 1.1.10, toast 1.2.15, tooltip 1.2.8
- UI/UX: sonner 2.0.7, lucide-react 0.562.0, class-variance-authority 0.7.1, tailwind-merge 3.4.0
- Animações: framer-motion 12.27.0
- Tauri/API: @tauri-apps/api 2.10.2, @tauri-apps/plugin-opener 2.10.2
- Dev: @tauri-apps/cli 2.10.2, @vitejs/plugin-react 4.6.0, vite 7.0.4, typescript ~5.8.3, @types/react 19.1.8, @types/react-dom 19.1.6

## Padrões Arquiteturais (OBRIGATÓRIOS)

### Backend (Rust/Tauri)

**Estrutura de Módulos - DEVE seguir:**
```
api/
├── {resource}/
│   ├── mod.rs              # Exporta sub-módulos
│   ├── {resource}_model.rs # Models (Input, Output, DB, Public)
│   ├── {resource}_query.rs # TODAS queries SQL (separação obrigatória)
│   ├── {resource}_handler.rs # Lógica de negócio (NUNCA SQL direto)
│   ├── {resource}_tool.rs  # Utilitários (hash, keys, etc)
│   └── {resource}_validator.rs # Validações reutilizáveis
```

**Separação de Responsabilidades - OBRIGATÓRIO:**
- **Handlers (`*_handler.rs`)**: Contêm APENAS lógica de negócio. **NUNCA** executam SQL diretamente.
- **Queries (`*_query.rs`)**: Contêm TODAS as queries SQL. Handlers chamam queries, nunca SQL direto.
- **Models (`*_model.rs`)**: Definem estruturas de dados (Input, Output, DB, Public).
- **Validators (`*_validator.rs`)**: Funções de validação reutilizáveis (ex: `validate_password_strength`).

**Padrão "Public" - OBRIGATÓRIO:**
- **NUNCA** exponha IDs internos do banco (`id: i64`) ao frontend.
- **SEMPRE** use UUID (`uuid: String`) para identificação pública.
- Models expostos ao frontend **DEVEM** ter sufixo `Public` (ex: `LocationPublic`, `ValidateResetTokenPublic`).
- Models internos **NÃO** devem ter `Serialize` se contiverem IDs internos.

**Configuração - OBRIGATÓRIO:**
- Use `OnceLock<T>` para configurações singleton (ex: `AuthConfig`, `PasswordResetConfig`).
- Função `get_*_config()` retorna referência estática.
- Hardcoded defaults + override opcional via JSON em `app_config_dir`.

**Respostas de API - OBRIGATÓRIO:**
- **SEMPRE** retorne `ApiResponse<T>` para sucesso ou `ApiError` para erro.
- Use `ApiResponse::ok(data)` para sucesso.
- Use `ApiError::err(message)` para erros.

**Autenticação - OBRIGATÓRIO:**
- **SEMPRE** use `validate_bearer(token)` em handlers que requerem autenticação.
- Retorna `AuthContext` com `user_uuid` e `email`.
- Handlers autenticados recebem `token: String` como primeiro parâmetro.

**Tracing - OBRIGATÓRIO:**
- **SEMPRE** use `#[instrument]` em handlers e queries.
- **SEMPRE** gere `request_id` (UUID) em Tauri commands.
- **SEMPRE** crie span com `info_span!` incluindo `request_id`.
- Log sucesso com `info!`, erros com `error!`.

**Validação - OBRIGATÓRIO:**
- Validações reutilizáveis **DEVEM** estar em `*_validator.rs`.
- Models de input **DEVEM** ter método `validate()`.
- Validação frontend (tempo real) + backend (obrigatória).

**Tauri Commands - OBRIGATÓRIO:**
- **SEMPRE** gere `request_id` (UUID).
- **SEMPRE** crie span de tracing.
- **SEMPRE** retorne `Result<ApiResponse<T>, ApiError>`.
- **SEMPRE** log sucesso/erro com `request_id`.

### Frontend (React/TypeScript)

**Estrutura de Pastas - DEVE seguir:**
```
src/
├── pages/           # Páginas/rotas
├── components/       # Componentes reutilizáveis
│   ├── auth/        # Componentes de autenticação
│   └── ui/          # Componentes UI base (Radix)
├── services/        # Chamadas Tauri (invoke)
├── types/           # TypeScript types
└── context/         # React Context (Auth, etc)
```

**Chamadas Tauri - OBRIGATÓRIO:**
- **NUNCA** faça chamadas HTTP diretas do frontend.
- **SEMPRE** use `invoke<T>()` do `@tauri-apps/api`.
- **SEMPRE** tipar retorno com `ApiResponse<T>`.
- Services em `src/services/*Api.ts` (ex: `authApi.ts`).

**Tipagem - OBRIGATÓRIO:**
- **NUNCA** use `any`. Use `unknown` se necessário.
- **SEMPRE** tipar payloads e responses.
- Types em `src/types/*.ts` correspondentes aos models Rust.

**Validação Frontend - OBRIGATÓRIO:**
- Validação em tempo real com feedback visual.
- Indicadores visuais (ícones, cores) durante digitação.
- Desabilitar submit até validação completa.
- **SEMPRE** validar também no backend (não confiar apenas no frontend).

**Feedback ao Usuário - OBRIGATÓRIO:**
- **SEMPRE** usar `sonner` para toasts (sucesso/erro).
- **SEMPRE** mostrar loaders durante operações assíncronas.
- **SEMPRE** desabilitar inputs/botões durante loading.

**Rotas - OBRIGATÓRIO:**
- Rotas públicas: `/login`, `/register`, `/forgot-password`, `/reset-password`.
- Rotas protegidas: **SEMPRE** usar `<ProtectedRoute>` wrapper.
- Layouts compartilhados: `AuthLayout` para páginas de auth.

## Padrões e Guias (OBRIGATÓRIOS)

**Nomenclatura - OBRIGATÓRIO:**
- Textos, comentários e nomes de variáveis **SEMPRE** em inglês.
- Arquivos Rust: `snake_case` (ex: `auth_handler.rs`).
- Arquivos TypeScript: `PascalCase` para componentes, `camelCase` para utils.
- Structs Rust: `PascalCase` (ex: `LoginInput`).
- Funções: `snake_case` em Rust, `camelCase` em TypeScript.

**Componentes React - OBRIGATÓRIO:**
- Componentes pequenos, nomes claros, sem efeitos colaterais fora de hooks.
- **SEMPRE** usar `motion` do framer-motion para animações de entrada.
- **SEMPRE** usar `AuthLayout` para páginas de autenticação.

**Hooks e Services - OBRIGATÓRIO:**
- Hooks reutilizáveis para lógica de dados/estado.
- **SEMPRE** separar camadas: services para chamadas Tauri.
- **SEMPRE** usar `useAuth()` para autenticação.

**Acessibilidade - OBRIGATÓRIO:**
- **SEMPRE** usar Radix primitives (acessíveis por padrão).
- **SEMPRE** incluir `aria-label` em botões sem texto.
- **SEMPRE** usar `sr-only` para texto apenas para screen readers.
- **SEMPRE** garantir foco visível em elementos interativos.

**Temas - OBRIGATÓRIO:**
- **SEMPRE** usar `next-themes` para gerenciamento de tema.
- **APENAS UM** toggle de tema por layout (na NavBar).
- **NUNCA** duplicar toggles de tema.

**Estilo - OBRIGATÓRIO:**
- **SEMPRE** usar Tailwind com tokens CSS vars.
- **SEMPRE** usar `cva` para variantes de componentes.
- **SEMPRE** usar `tailwind-merge` para mesclar classes.
- **SEMPRE** usar classes utilitárias do Tailwind, evitar CSS customizado.

**Feedback - OBRIGATÓRIO:**
- **SEMPRE** usar toasts Sonner para estados de rede (sucesso/erro).
- **SEMPRE** mostrar loaders consistentes durante operações assíncronas.
- **SEMPRE** usar ícones Lucide para feedback visual.

**Configuração e Build - OBRIGATÓRIO:**
- **SEMPRE** checar lint/TS antes de commits (`yarn build` deve passar).
- **SEMPRE** revisar permissões Tauri em `tauri.conf.json`.
- **SEMPRE** testar em dev antes de commit (`yarn tauri dev`).

**Segurança - OBRIGATÓRIO:**
- **NUNCA** expor IDs internos do banco ao frontend.
- **SEMPRE** usar UUID para identificação pública.
- **SEMPRE** validar inputs no backend (não confiar apenas no frontend).
- **SEMPRE** usar `validate_bearer()` para rotas protegidas.
- **SEMPRE** hash senhas com `scrypt` (nunca armazenar em texto plano).

