//! Scheduler de trabalhos agendados (Fase B.1): cron de 5 campos
//! (`minuto hora dia-do-mês mês dia-da-semana`) no fuso
//! `America/Sao_Paulo`, com suporte a `*`, `*/n`, listas e intervalos.

use chrono::{DateTime, Datelike, TimeZone, Utc};

use crate::domain::trabalhos::ports::SchedulerTrabalho;

/// Anos de busca máxima por uma próxima execução — um cron válido sempre
/// casa em até um ano (considerando 29 de fevereiro em ano bissexto,
/// usamos uma margem folgada).
const ANOS_BUSCA: i32 = 5;

/// Scheduler com relógio de parede no fuso de Brasília.
#[derive(Debug, Clone, Copy, Default)]
pub struct SchedulerTrabalhoCron;

impl SchedulerTrabalhoCron {
    pub const fn novo() -> Self {
        Self
    }
}

struct CamposCron {
    minutos: Vec<u32>,
    horas: Vec<u32>,
    dias: Vec<u32>,
    meses: Vec<u32>,
    dias_semana: Vec<u32>,
}

impl CamposCron {
    fn analisar(expressao: &str) -> Option<Self> {
        let partes: Vec<&str> = expressao.split_whitespace().collect();
        if partes.len() != 5 {
            return None;
        }
        let mut dias_semana = analisar_campo(partes[4], 0, 7)?;
        // 7 é domingo (alias de 0), como no cron clássico.
        for dia in &mut dias_semana {
            if *dia == 7 {
                *dia = 0;
            }
        }
        dias_semana.sort_unstable();
        dias_semana.dedup();

        Some(Self {
            minutos: analisar_campo(partes[0], 0, 59)?,
            horas: analisar_campo(partes[1], 0, 23)?,
            dias: analisar_campo(partes[2], 1, 31)?,
            meses: analisar_campo(partes[3], 1, 12)?,
            dias_semana,
        })
    }
}

/// Analisa um campo do cron: `*`, `*/n`, `a`, `a-b`, `a-b/n` e listas
/// separadas por vírgula, sempre dentro de `[minimo, maximo]`.
fn analisar_campo(campo: &str, minimo: u32, maximo: u32) -> Option<Vec<u32>> {
    let mut valores = Vec::new();
    for parte in campo.split(',') {
        let parte = parte.trim();
        if parte.is_empty() {
            return None;
        }
        let (base, passo) = match parte.split_once('/') {
            Some((base, passo)) => {
                let passo = passo.parse::<u32>().ok().filter(|passo| *passo > 0)?;
                (base, Some(passo))
            }
            None => (parte, None),
        };
        let (inicio, fim) = if base == "*" {
            (minimo, maximo)
        } else if let Some((inicio, fim)) = base.split_once('-') {
            let inicio = inicio.parse::<u32>().ok()?;
            let fim = fim.parse::<u32>().ok()?;
            if inicio > fim || inicio < minimo || fim > maximo {
                return None;
            }
            (inicio, fim)
        } else {
            let valor = base.parse::<u32>().ok()?;
            if !(minimo..=maximo).contains(&valor) {
                return None;
            }
            (valor, valor)
        };
        let passo = passo.unwrap_or(1);
        let mut valor = inicio;
        while valor <= fim {
            valores.push(valor);
            valor += passo;
        }
    }
    if valores.is_empty() {
        return None;
    }
    valores.sort_unstable();
    valores.dedup();
    Some(valores)
}

impl SchedulerTrabalho for SchedulerTrabalhoCron {
    fn proxima_execucao(&self, cron: &str, depois_de: DateTime<Utc>) -> Option<DateTime<Utc>> {
        let campos = CamposCron::analisar(cron)?;
        let fuso: chrono_tz::Tz = chrono_tz::America::Sao_Paulo;
        let local = depois_de.with_timezone(&fuso);
        let mut dia = local.date_naive();

        for _ in 0..(ANOS_BUSCA * 366) {
            let dia_da_semana = dia.weekday().num_days_from_sunday();
            let mes = dia.month();
            let dia_do_mes = dia.day();

            if campos.meses.contains(&mes)
                && campos.dias.contains(&dia_do_mes)
                && campos.dias_semana.contains(&dia_da_semana)
            {
                for hora in &campos.horas {
                    for minuto in &campos.minutos {
                        let naive = dia.and_hms_opt(*hora, *minuto, 0)?;
                        // `earliest()` escolhe o primeiro instante válido em
                        // fusos com horário de verão; America/Sao_Paulo não
                        // tem DST desde 2019, mas a regra fica correta.
                        if let Some(candidato) = fuso.from_local_datetime(&naive).earliest() {
                            let candidato_utc = candidato.with_timezone(&Utc);
                            if candidato_utc > depois_de {
                                return Some(candidato_utc);
                            }
                        }
                    }
                }
            }

            dia = dia.succ_opt()?;
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(ano: i32, mes: u32, dia: u32, hora: u32, minuto: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(ano, mes, dia, hora, minuto, 0)
            .single()
            .expect("data válida")
    }

    #[test]
    fn diaria_no_fuso_de_brasilia() {
        let scheduler = SchedulerTrabalhoCron::novo();
        // 2026-11-11 12:00 UTC = 09:00 em São Paulo; próxima 03:00 BRT
        // (06:00 UTC) é no dia seguinte.
        let proxima = scheduler
            .proxima_execucao("0 3 * * *", utc(2026, 11, 11, 12, 0))
            .expect("cron válido");
        assert_eq!(proxima, utc(2026, 11, 12, 6, 0));
    }

    #[test]
    fn semanal_na_segunda_as_nove() {
        let scheduler = SchedulerTrabalhoCron::novo();
        // 2026-11-11 é quarta-feira. Segunda seguinte: 2026-11-16.
        let proxima = scheduler
            .proxima_execucao("0 9 * * 1", utc(2026, 11, 11, 12, 0))
            .expect("cron válido");
        assert_eq!(proxima, utc(2026, 11, 16, 12, 0));
    }

    #[test]
    fn respeita_expressao_estritamente_posterior() {
        let scheduler = SchedulerTrabalhoCron::novo();
        // Exatamente às 09:00 BRT de 11/11 — a próxima é 12/11.
        let proxima = scheduler
            .proxima_execucao("0 9 * * *", utc(2026, 11, 11, 12, 0))
            .expect("cron válido");
        assert_eq!(proxima, utc(2026, 11, 12, 12, 0));
    }

    #[test]
    fn intervalos_e_listas() {
        let scheduler = SchedulerTrabalhoCron::novo();
        // A cada 15 minutos, das 8h às 10h, de segunda a sexta.
        let base = utc(2026, 11, 11, 11, 7); // 08:07 BRT
        let proxima = scheduler
            .proxima_execucao("*/15 8-10 * * 1-5", base)
            .expect("cron válido");
        assert_eq!(proxima, utc(2026, 11, 11, 11, 15));
    }

    #[test]
    fn rejeita_cron_invalido() {
        let scheduler = SchedulerTrabalhoCron::novo();
        let base = utc(2026, 11, 11, 12, 0);
        for cron in [
            "",
            "* * * *",
            "60 * * * *",
            "* 24 * * *",
            "* * 0 * *",
            "* * * 13 *",
            "* * * * 8",
            "*/0 * * * *",
            "a b c d e",
        ] {
            assert!(
                scheduler.proxima_execucao(cron, base).is_none(),
                "cron deveria ser inválido: {cron:?}"
            );
        }
    }

    #[test]
    fn domingo_aceita_0_e_7() {
        let scheduler = SchedulerTrabalhoCron::novo();
        let base = utc(2026, 11, 11, 12, 0); // quarta
        let com_zero = scheduler
            .proxima_execucao("0 9 * * 0", base)
            .expect("cron válido");
        let com_sete = scheduler
            .proxima_execucao("0 9 * * 7", base)
            .expect("cron válido");
        assert_eq!(com_zero, com_sete);
        assert_eq!(com_zero, utc(2026, 11, 15, 12, 0)); // domingo
    }
}
