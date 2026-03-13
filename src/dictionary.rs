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
}
