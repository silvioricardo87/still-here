use crate::config::Language;
use rand::seq::SliceRandom;

// ---------------------------------------------------------------------------
// Portuguese (pt-br) phrases — 120+ entries
// Themes: email openings/closings, meeting notes, project updates,
//         sprint reviews, status reports, task descriptions
// ---------------------------------------------------------------------------

const PHRASES_PT_BR: &[&str] = &[
    // Email openings
    "Bom dia, seguem as atualizações do projeto conforme solicitado.",
    "Prezados, encaminho abaixo o resumo da reunião de ontem.",
    "Olá equipe, gostaria de compartilhar o status atual das atividades.",
    "Boa tarde, segue o relatório de progresso da sprint atual.",
    "Caro time, informo que as entregas da semana estão sendo finalizadas.",
    "Olá a todos, seguem os pontos discutidos na reunião de alinhamento.",
    "Bom dia, conforme combinado, envio o sumário executivo do projeto.",
    "Prezado gestor, segue a atualização das tarefas em andamento.",
    "Olá, estou encaminhando o resumo das atividades realizadas hoje.",
    "Boa tarde equipe, veja abaixo o status das demandas pendentes.",
    // Email closings
    "Fico à disposição para esclarecimentos adicionais.",
    "Qualquer dúvida, estou disponível para uma conversa rápida.",
    "Aguardo retorno sobre os pontos levantados.",
    "Em caso de dúvidas, não hesite em me contatar.",
    "Permanecemos à disposição para alinhamentos necessários.",
    "Caso precise de mais informações, é só me chamar.",
    "Atenciosamente e aguardando retorno.",
    "Obrigado pela atenção e colaboração de todos.",
    "Continuamos acompanhando os desdobramentos e reportamos em breve.",
    "Grato pela atenção e fico no aguardo do feedback.",
    // Meeting notes
    "Reunião de alinhamento realizada com todos os stakeholders presentes.",
    "Foram discutidos os bloqueios da sprint e definidos planos de ação.",
    "O time concordou em revisar o backlog na próxima semana.",
    "Decisão tomada: manter o escopo atual e renegociar o prazo.",
    "Action items distribuídos e registrados no board do projeto.",
    "Próxima reunião agendada para quinta-feira às dez horas.",
    "Levantados os riscos do projeto e definidas mitigações.",
    "Retrospectiva realizada; pontos de melhoria identificados.",
    "Daily concluída sem bloqueios críticos relatados.",
    "Revisão do roadmap feita e prioridades reajustadas.",
    "Acordado que as entregas serão validadas até o final da semana.",
    "Planning realizado com estimativas em story points para cada história.",
    "Sprint review apresentada ao cliente com feedback positivo.",
    "Definida a meta da sprint: entregar o módulo de autenticação.",
    "Foram identificados três impedimentos que precisam de resolução urgente.",
    // Project updates
    "O módulo de integração foi concluído e está em homologação.",
    "A feature de relatórios está em desenvolvimento, previsão para sexta.",
    "Testes de regressão iniciados após a última entrega.",
    "Pipeline de CI/CD configurado e funcionando corretamente.",
    "Documentação técnica atualizada com os novos endpoints.",
    "Refatoração do componente de autenticação concluída.",
    "Deploy realizado em ambiente de staging para validação.",
    "Revisão de código completa; três PRs aprovados hoje.",
    "Banco de dados migrado com sucesso para a nova estrutura.",
    "Performance do sistema melhorada em quarenta por cento após otimizações.",
    "Cobertura de testes unitários atingiu oitenta e cinco por cento.",
    "Integração com a API de pagamentos homologada e aprovada.",
    "Correção do bug crítico em produção realizada com sucesso.",
    "Nova versão do sistema publicada após aprovação do cliente.",
    "Configurações de segurança revisadas e atualizadas.",
    // Sprint reviews
    "Sprint encerrada com noventa por cento das histórias entregues.",
    "Velocity da equipe manteve-se estável nas últimas três sprints.",
    "Dívida técnica reduzida após refatoração realizada nesta sprint.",
    "Meta de entrega atingida: todos os critérios de aceitação validados.",
    "Histórias não concluídas foram reestimadas e movidas para o próximo ciclo.",
    "Demonstração ao cliente realizada com aprovação de todas as funcionalidades.",
    "Burndown da sprint mostra progresso consistente ao longo do ciclo.",
    "Time completou doze pontos acima do planejado nesta sprint.",
    "Funcionalidades entregues foram validadas pelo time de QA.",
    "Sprint retrospectiva apontou necessidade de melhorar comunicação assíncrona.",
    // Status reports
    "Status atual: em andamento, sem bloqueios identificados.",
    "Progresso: setenta por cento concluído, dentro do prazo previsto.",
    "Situação: aguardando aprovação do cliente para prosseguir.",
    "Andamento: fase de testes iniciada, previsão de conclusão na sexta.",
    "Atualização: entrega realizada conforme cronograma estabelecido.",
    "Status: tarefa concluída e validada pelo responsável.",
    "Progresso: aguardando dependência externa para finalizar.",
    "Situação crítica: bloqueio identificado, time trabalhando na solução.",
    "Atualização diária: atividades dentro do planejado.",
    "Relatório semanal: todas as metas atingidas no período.",
    // Task descriptions
    "Implementar endpoint de listagem de usuários com paginação.",
    "Corrigir bug de cálculo no módulo financeiro reportado pelo cliente.",
    "Criar tela de cadastro de produtos com validação de formulário.",
    "Refatorar serviço de envio de e-mails para usar fila de mensagens.",
    "Atualizar dependências do projeto para versões mais recentes.",
    "Escrever testes unitários para o módulo de autenticação.",
    "Configurar monitoramento de erros com alertas automáticos.",
    "Revisar e atualizar a documentação da API REST.",
    "Criar script de migração para atualizar estrutura do banco.",
    "Implementar cache Redis para melhorar performance das consultas.",
    "Analisar logs de produção e identificar gargalos de performance.",
    "Configurar pipeline de deploy automatizado para ambiente de staging.",
    "Desenvolver relatório exportável em formato PDF e Excel.",
    "Integrar sistema com serviço externo de autenticação SSO.",
    "Criar dashboard de métricas para acompanhamento em tempo real.",
    // Miscellaneous office / collaboration phrases
    "Atualizei o quadro de tarefas com o progresso de hoje.",
    "Documentei a decisão no Confluence para referência futura.",
    "Criei o ticket no Jira com todos os detalhes necessários.",
    "Compartilhei o link da gravação da reunião no canal do Teams.",
    "Fiz a revisão do documento e deixei comentários inline.",
    "Atualizei o cronograma com as novas datas acordadas.",
    "Reservei a sala de reunião para o alinhamento de amanhã.",
    "Enviei o convite de calendário para todos os participantes.",
    "Preparei a apresentação para a reunião com a diretoria.",
    "Atualizei o README com as instruções de configuração do ambiente.",
    "Realizei testes manuais e confirmei que o comportamento está correto.",
    "Fiz o merge da branch após aprovação de dois revisores.",
    "Resolvi os conflitos de merge e atualizei a pull request.",
    "Ajustei as configurações de ambiente conforme orientação da infra.",
    "Validei os dados migrados comparando com a base antiga.",
    "Criei os casos de teste baseados nos critérios de aceite.",
    "Alinhei com o PO sobre as prioridades para a próxima sprint.",
    "Participei da sessão de refinamento e contribuí com estimativas.",
    "Levantei os requisitos faltantes junto ao cliente.",
    "Elaborei o diagrama de fluxo do novo processo de negócio.",
    "Registrei o incidente e abri chamado com a equipe de infraestrutura.",
    "Atualizei o plano de testes com os novos cenários de uso.",
    "Configurei as variáveis de ambiente no servidor de homologação.",
    "Revisei o código do colega e aprovei a pull request.",
    "Participei do onboarding do novo membro da equipe.",
    "Identifiquei a causa raiz do problema e propus solução.",
    "Realizei a análise de impacto antes de iniciar a mudança.",
    "Documentei o procedimento de rollback em caso de falha.",
    "Atualizei o status da tarefa para em revisão no board.",
    "Coordenei a resolução do incidente em produção.",
    // Programming — code snippets and syntax
    "fn main() { let args: Vec<String> = std::env::args().collect(); }",
    "pub struct Config { pub host: String, pub port: u16, pub debug: bool }",
    "impl Default for Config { fn default() -> Self { Config { host: String::from(\"localhost\"), port: 8080, debug: false } } }",
    "let result = match response.status() { 200 => Ok(body), 404 => Err(\"Not found\"), _ => Err(\"Unknown error\") };",
    "async fn fetch_data(url: &str) -> Result<Response, reqwest::Error> { reqwest::get(url).await }",
    "use std::collections::HashMap; let mut map = HashMap::new(); map.insert(\"key\", \"value\");",
    "#[derive(Debug, Clone, Serialize, Deserialize)] pub struct User { pub id: u64, pub name: String, pub email: String }",
    "impl fmt::Display for AppError { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, \"{}\", self.message) } }",
    "let items: Vec<i32> = data.iter().filter(|x| x.is_valid()).map(|x| x.value).collect();",
    "if let Some(config) = load_config() { println!(\"Config loaded: {:?}\", config); }",
    "pub async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> { Ok(Response::new(Body::from(\"OK\"))) }",
    "fn calculate_hash(input: &[u8]) -> u64 { let mut hasher = DefaultHasher::new(); input.hash(&mut hasher); hasher.finish() }",
    "const MAX_RETRIES: u32 = 3; const TIMEOUT_MS: u64 = 5000; const BUFFER_SIZE: usize = 4096;",
    "enum State { Idle, Running(u32), Paused, Error(String) }",
    "trait Repository<T> { fn find_by_id(&self, id: u64) -> Option<T>; fn save(&mut self, entity: T) -> Result<(), Error>; }",
    "let connection_string = format!(\"postgres://{}:{}@{}:{}/{}\", user, pass, host, port, db);",
    "pub fn parse_json<T: DeserializeOwned>(input: &str) -> Result<T, serde_json::Error> { serde_json::from_str(input) }",
    "#[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> { let app = Router::new().route(\"/api\", get(handler)); }",
    "let mut file = File::create(\"output.txt\")?; writeln!(file, \"{}\", data)?;",
    "spawn(move || { loop { let msg = rx.recv().unwrap(); process(msg); } });",
    // Programming — terminal and git commands
    "git checkout -b feature/user-authentication origin/develop",
    "git rebase -i HEAD~5 && git push --force-with-lease origin feature/api-refactor",
    "git stash push -m \"work in progress on login module\" && git pull --rebase origin main",
    "docker compose up -d --build && docker logs -f api-service",
    "cargo test --workspace -- --nocapture 2>&1 | tee test_output.log",
    "kubectl get pods -n production | grep -v Running | awk '{print $1}'",
    "curl -X POST http://localhost:3000/api/users -H 'Content-Type: application/json' -d '{\"name\": \"test\"}'",
    "npm run build && npm run test:coverage && npm run lint:fix",
    "psql -U admin -d appdb -c 'SELECT COUNT(*) FROM users WHERE active = true;'",
    "ssh deploy@server.prod.internal 'cd /opt/app && ./deploy.sh v2.4.1'",
    "grep -rn 'TODO\\|FIXME\\|HACK' src/ --include='*.rs' | sort",
    "find . -name '*.log' -mtime +30 -exec rm {} \\;",
    "python -m pytest tests/ -v --cov=src --cov-report=html",
    "terraform plan -var-file=production.tfvars -out=plan.tfplan",
    "aws s3 sync ./dist s3://app-bucket/static --delete --cache-control max-age=31536000",
    // Programming — SQL queries
    "SELECT u.name, COUNT(o.id) AS total_orders FROM users u LEFT JOIN orders o ON u.id = o.user_id GROUP BY u.name HAVING COUNT(o.id) > 5 ORDER BY total_orders DESC;",
    "CREATE INDEX CONCURRENTLY idx_users_email ON users (email) WHERE deleted_at IS NULL;",
    "INSERT INTO audit_log (user_id, action, timestamp) VALUES ($1, $2, NOW()) RETURNING id;",
    "UPDATE products SET price = price * 1.10, updated_at = NOW() WHERE category_id = 3 AND active = true;",
    "WITH monthly_stats AS (SELECT date_trunc('month', created_at) AS month, SUM(amount) AS total FROM transactions GROUP BY 1) SELECT * FROM monthly_stats ORDER BY month DESC LIMIT 12;",
    "ALTER TABLE orders ADD COLUMN tracking_code VARCHAR(64), ADD COLUMN shipped_at TIMESTAMP;",
    "DELETE FROM sessions WHERE last_active < NOW() - INTERVAL '30 days';",
    "EXPLAIN ANALYZE SELECT * FROM products WHERE name ILIKE '%search_term%' AND stock > 0 ORDER BY created_at DESC LIMIT 20;",
    // Programming — code comments and documentation
    "// TODO: refatorar este método para usar async/await em vez de callbacks",
    "// FIXME: race condition quando dois threads acessam o cache simultaneamente",
    "/// Processa a fila de mensagens e retorna o número de itens processados com sucesso.",
    "// Nota: este endpoint requer autenticação via Bearer token no header Authorization",
    "// Workaround para o bug #4521 — remover quando a lib for atualizada para v3.x",
    "/// Calcula o hash SHA-256 do payload e compara com a assinatura fornecida no header.",
    "// HACK: usando sleep(100ms) porque a API externa retorna 429 se chamar muito rápido",
    "/// Retorna uma conexão do pool. Falha com timeout após 5 segundos se o pool estiver esgotado.",
    // Programming — config files and snippets
    "[dependencies]\ntokio = { version = \"1\", features = [\"full\"] }\nserde = { version = \"1\", features = [\"derive\"] }\nreqwest = { version = \"0.11\", features = [\"json\"] }",
    "server { listen 443 ssl; server_name api.example.com; location /api { proxy_pass http://127.0.0.1:8080; } }",
    "services:\n  api:\n    image: app:latest\n    ports:\n      - \"8080:8080\"\n    environment:\n      - DATABASE_URL=postgres://db:5432/app",
    "FROM rust:1.75 AS builder\nWORKDIR /app\nCOPY . .\nRUN cargo build --release\nFROM debian:bookworm-slim\nCOPY --from=builder /app/target/release/server /usr/local/bin/",
    "{\n  \"compilerOptions\": {\n    \"target\": \"ES2022\",\n    \"module\": \"esnext\",\n    \"strict\": true,\n    \"outDir\": \"./dist\"\n  }\n}",
    // Programming — developer thoughts and logs
    "Preciso revisar o handler de erros, ele não está retornando o status code correto para 422.",
    "O teste de integração está falhando porque o mock do banco não reseta entre os cenários.",
    "A query de relatório está demorando mais de três segundos, preciso adicionar um índice composto.",
    "Vou extrair essa lógica de validação para um middleware separado para reutilizar nos outros endpoints.",
    "O memory leak aparece depois de processar cerca de dez mil requisições, provavelmente no connection pool.",
    "Descobri que o deadlock acontece quando o lock A é adquirido antes do lock B na thread secundária.",
    "Preciso migrar esse serviço de REST para gRPC para reduzir a latência entre os microsserviços.",
    "A cobertura de testes caiu para setenta por cento depois do merge, preciso adicionar testes para o novo módulo.",
    "O CI está quebrando por causa de uma dependência transitiva que foi removida do registry.",
    "Vou usar feature flags para fazer o deploy gradual da nova funcionalidade sem impactar todos os usuários.",
];

// ---------------------------------------------------------------------------
// English phrases — 60+ entries
// Themes: email openings/closings, meeting notes, project updates,
//         sprint reviews, status reports, task descriptions
// ---------------------------------------------------------------------------

const PHRASES_EN: &[&str] = &[
    // Email openings
    "Hi team, please find below the project status update for this week.",
    "Good morning, I wanted to share a quick progress report on current tasks.",
    "Hello everyone, following up on yesterday's meeting discussion points.",
    "Hi all, attaching the sprint summary as requested by the stakeholders.",
    "Good afternoon, here is the weekly status report for your review.",
    "Hi, just wanted to provide an update on the ongoing deliverables.",
    "Hello team, sharing the retrospective notes from today's session.",
    "Dear stakeholders, please see the project milestone update below.",
    // Email closings
    "Let me know if you have any questions or need further clarification.",
    "Feel free to reach out if you need additional information.",
    "Looking forward to your feedback on the points mentioned above.",
    "Please don't hesitate to contact me if anything is unclear.",
    "Thanks for your time and collaboration on this initiative.",
    "Happy to jump on a quick call to discuss further if needed.",
    "Appreciate your continued support on this project.",
    "Thank you and looking forward to our next sync.",
    // Meeting notes
    "Sprint planning completed; team committed to twelve story points.",
    "Retrospective held; action items assigned to respective owners.",
    "Daily standup completed with no blockers reported by the team.",
    "Stakeholder review meeting concluded with positive feedback.",
    "Decision made to defer non-critical features to the next release.",
    "Risk assessment completed and mitigation plans documented.",
    "Roadmap reviewed and priorities realigned based on business needs.",
    "Backlog refinement session completed with updated story estimates.",
    "Architecture review meeting produced a consensus on the new design.",
    "Cross-team sync completed; dependencies identified and tracked.",
    // Project updates
    "Authentication module deployed to staging and pending QA sign-off.",
    "API integration completed and ready for end-to-end testing.",
    "Database migration executed successfully with zero downtime.",
    "CI/CD pipeline updated to include automated security scans.",
    "Performance benchmarks show a thirty percent improvement after optimization.",
    "Unit test coverage increased to eighty-eight percent this sprint.",
    "Critical production bug fixed and hotfix deployed within SLA.",
    "Feature branch merged after review approval from two team members.",
    "Technical documentation updated to reflect the latest API changes.",
    "Third-party library upgraded to address known security vulnerabilities.",
    // Sprint reviews
    "Sprint closed with ninety-five percent of committed stories delivered.",
    "Team velocity remained consistent over the last four sprints.",
    "All acceptance criteria met and validated by the product owner.",
    "Incomplete stories re-estimated and moved to the upcoming sprint.",
    "Client demo conducted successfully with approval on all delivered features.",
    "Burndown chart shows steady progress throughout the sprint cycle.",
    "Technical debt reduced significantly through targeted refactoring efforts.",
    "Sprint goal achieved ahead of schedule, allowing for buffer tasks.",
    // Status reports
    "Current status: in progress, on track for the planned delivery date.",
    "Progress: sixty percent complete, no blockers at this time.",
    "Status: awaiting client approval before proceeding to the next phase.",
    "Update: task completed and verified by the QA team.",
    "Situation: blocked on external dependency, escalation in progress.",
    "Weekly report: all milestones met as per the project plan.",
    "Daily update: activities proceeding as planned without issues.",
    // Task descriptions
    "Implement user authentication using OAuth 2.0 and JWT tokens.",
    "Fix pagination bug in the product listing endpoint.",
    "Write integration tests for the payment processing workflow.",
    "Set up automated alerts for critical error thresholds in production.",
    "Refactor the notification service to use an event-driven architecture.",
    "Create an exportable PDF report for monthly sales data.",
    "Update environment variables in the staging deployment configuration.",
    "Review and merge open pull requests before the end of the sprint.",
    "Investigate performance degradation reported in the latest load test.",
    "Document the deployment runbook for the upcoming release.",
    // Programming — code snippets and syntax
    "fn process_batch(items: &[Record]) -> Vec<Result<(), Error>> { items.par_iter().map(|i| validate_and_save(i)).collect() }",
    "const app = express(); app.use(cors()); app.use(express.json()); app.listen(3000);",
    "export async function fetchUsers(): Promise<User[]> { const res = await fetch('/api/users'); return res.json(); }",
    "interface ApiResponse<T> { data: T; status: number; message: string; timestamp: Date; }",
    "class UserService { constructor(private readonly repo: UserRepository, private readonly cache: CacheService) {} }",
    "def process_events(queue: Queue) -> None:\n    while event := queue.get():\n        handler = HANDLERS.get(event.type)\n        if handler: handler(event)",
    "SELECT p.name, c.name AS category, p.price FROM products p INNER JOIN categories c ON p.category_id = c.id WHERE p.price BETWEEN 10 AND 100 ORDER BY p.price;",
    "const pipeline = [ { $match: { status: 'active' } }, { $group: { _id: '$category', total: { $sum: '$amount' } } }, { $sort: { total: -1 } } ];",
    "func (s *Server) HandleRequest(w http.ResponseWriter, r *http.Request) { json.NewEncoder(w).Encode(response) }",
    "public record UserDto(Long id, String name, String email, LocalDateTime createdAt) {}",
    "@app.route('/api/health', methods=['GET'])\ndef health_check():\n    return jsonify({'status': 'ok', 'uptime': get_uptime()}), 200",
    "CREATE TABLE IF NOT EXISTS migrations (id SERIAL PRIMARY KEY, name VARCHAR(255) NOT NULL, applied_at TIMESTAMP DEFAULT NOW());",
    "impl<T: Send + Sync> From<tokio::sync::mpsc::error::SendError<T>> for AppError { fn from(e: SendError<T>) -> Self { AppError::Channel(e.to_string()) } }",
    "type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;",
    "let schema = z.object({ email: z.string().email(), password: z.string().min(8), role: z.enum(['admin', 'user']) });",
    "const [state, dispatch] = useReducer(reducer, { items: [], loading: false, error: null });",
    "pub fn middleware(req: ServiceRequest, srv: &AppService) -> impl Future<Output = Result<ServiceResponse, Error>> { }",
    "docker build -t myapp:$(git rev-parse --short HEAD) . && docker push registry.io/myapp:$(git rev-parse --short HEAD)",
    "#[cfg(test)] mod tests { use super::*; use mockall::predicate::*; #[tokio::test] async fn test_create_user() { } }",
    "import { describe, it, expect, vi } from 'vitest'; describe('UserService', () => { it('should create user', async () => { }); });",
    // Programming — terminal and git commands
    "git log --oneline --graph --all --decorate | head -20",
    "git diff --stat HEAD~3..HEAD -- src/",
    "cargo clippy --all-targets --all-features -- -D warnings",
    "docker exec -it postgres-dev psql -U postgres -d myapp -c '\\dt+'",
    "kubectl apply -f k8s/deployment.yaml && kubectl rollout status deployment/api -n production",
    "redis-cli -h cache.internal MONITOR | grep -i 'SET\\|GET\\|DEL'",
    "jq '.results[] | {name: .name, status: .status}' api_response.json",
    "ab -n 10000 -c 100 -H 'Authorization: Bearer token123' http://localhost:8080/api/users",
    "openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes",
    "pg_dump --format=custom --compress=9 production_db > backup_$(date +%Y%m%d).dump",
    "tar czf deploy-$(date +%Y%m%d%H%M).tar.gz --exclude=node_modules --exclude=.git .",
    "wc -l src/**/*.rs | sort -n | tail -20",
    "yarn workspace @app/api add zod @types/node && yarn workspace @app/api tsc --noEmit",
    "git bisect start HEAD v2.3.0 -- src/auth/ && git bisect run cargo test auth_module",
    // Programming — code comments and documentation
    "// TODO: replace this polling loop with a proper event-driven approach using channels",
    "// FIXME: this query does a full table scan on large datasets, needs composite index",
    "/// Validates the JWT token and extracts the user claims. Returns Unauthorized on invalid tokens.",
    "// NOTE: rate limiter uses a sliding window of 60 seconds with max 100 requests per IP",
    "// Temporary workaround for issue #2847 — upstream fix expected in next minor release",
    "/// Establishes a database connection pool with min 5, max 20 connections and 30s idle timeout.",
    "// PERF: reduced allocations by switching from String to &str in the hot path",
    "/// Retries the operation up to 3 times with exponential backoff starting at 100ms.",
    // Programming — config and infra snippets
    "name: CI\non: [push, pull_request]\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: cargo test --workspace",
    "resource \"aws_lambda_function\" \"api\" {\n  function_name = \"api-handler\"\n  runtime = \"provided.al2\"\n  handler = \"bootstrap\"\n  memory_size = 256\n  timeout = 30\n}",
    "upstream backend { server 127.0.0.1:8001; server 127.0.0.1:8002; } server { location /api { proxy_pass http://backend; } }",
    "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: api-server\nspec:\n  replicas: 3\n  template:\n    spec:\n      containers:\n        - name: api\n          image: app:latest",
    // Programming — developer thoughts and logs
    "Need to add retry logic to the external API call, it's timing out intermittently under load.",
    "The flaky test in CI is caused by a race condition in the shared test database setup.",
    "Profiling shows the serialization step takes forty percent of the request time, switching to simd-json.",
    "Going to split this monolithic handler into separate middleware layers for auth, validation, and logging.",
    "The connection pool is exhausted during peak traffic, need to increase max connections and add queue limit.",
    "Found the root cause: the cache TTL was set to zero in the staging config, bypassing it entirely.",
    "Migrating from callbacks to async/await reduced the nesting depth from seven levels to two.",
    "The load test showed P99 latency spiking to two seconds, likely due to GC pauses in the sidecar.",
    "After switching to a B-tree index the query dropped from twelve seconds to fifteen milliseconds.",
    "Need to implement circuit breaker pattern for the payment gateway to handle downstream failures gracefully.",
];

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Returns the full phrase list for the given language.
pub fn get_phrases(lang: Language) -> &'static [&'static str] {
    match lang {
        Language::PtBr => PHRASES_PT_BR,
        Language::En => PHRASES_EN,
    }
}

/// Picks one phrase at random from the list for the given language.
pub fn random_phrase(lang: Language) -> &'static str {
    let phrases = get_phrases(lang);
    let mut rng = rand::thread_rng();
    phrases.choose(&mut rng).copied().unwrap_or("")
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pt_br_dictionary_not_empty() {
        let phrases = get_phrases(Language::PtBr);
        assert!(phrases.len() >= 100);
    }

    #[test]
    fn test_en_dictionary_not_empty() {
        let phrases = get_phrases(Language::En);
        assert!(phrases.len() >= 50);
    }

    #[test]
    fn test_random_phrase_returns_different_results() {
        let phrases: Vec<&str> = (0..10).map(|_| random_phrase(Language::PtBr)).collect();
        let unique: std::collections::HashSet<&&str> = phrases.iter().collect();
        assert!(unique.len() >= 2);
    }

    #[test]
    fn test_random_phrase_en_returns_different_results() {
        let phrases: Vec<&str> = (0..10).map(|_| random_phrase(Language::En)).collect();
        let unique: std::collections::HashSet<&&str> = phrases.iter().collect();
        assert!(unique.len() >= 2);
    }

    #[test]
    fn test_random_phrase_returns_non_empty() {
        for _ in 0..50 {
            let phrase = random_phrase(Language::PtBr);
            assert!(!phrase.is_empty());
            let phrase = random_phrase(Language::En);
            assert!(!phrase.is_empty());
        }
    }

    #[test]
    fn test_all_phrases_are_non_empty() {
        for phrase in get_phrases(Language::PtBr) {
            assert!(!phrase.is_empty(), "Found empty pt-br phrase");
        }
        for phrase in get_phrases(Language::En) {
            assert!(!phrase.is_empty(), "Found empty en phrase");
        }
    }

    #[test]
    fn test_get_phrases_returns_correct_arrays() {
        let pt = get_phrases(Language::PtBr);
        let en = get_phrases(Language::En);
        // pt-br should have more phrases than en
        assert!(pt.len() > en.len());
    }

    #[test]
    fn test_no_duplicate_phrases() {
        let pt = get_phrases(Language::PtBr);
        let unique: std::collections::HashSet<&&str> = pt.iter().collect();
        assert_eq!(pt.len(), unique.len(), "Found duplicate pt-br phrases");

        let en = get_phrases(Language::En);
        let unique: std::collections::HashSet<&&str> = en.iter().collect();
        assert_eq!(en.len(), unique.len(), "Found duplicate en phrases");
    }
}
