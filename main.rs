use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    collections::{HashSet, HashMap},
    env,
};

struct Atomo {
    valor: Option<bool>,
}

#[derive(Clone)]
struct Clausula {
    literais: HashSet<i32>,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Verifica se o nome do arquivo foi passado
    let nome_arquivo = if args.len() > 1 {
        &args[1]
    } else {
        eprintln!("Uso: {} <nome_do_arquivo>", args[0]);
        std::process::exit(1);
    };

    let file = File::open(format!("{}.cnf", nome_arquivo)).expect("Erro ao abrir o arquivo");
    let reader = BufReader::new(file);
    let mut indice_linha = 1;
    let mut tem_p = false;

    let mut clausulas: Vec<Clausula> = Vec::new();
    let mut atomos: Vec<Atomo> = Vec::new();

    // Le arquivo
    for linha in reader.lines() {
        let linha_valida = linha
            .expect(&format!("Erro ao ler linha {}", indice_linha));

        let conteudo_linha: Vec<&str> = linha_valida
            .split_whitespace()
            .collect();

        if conteudo_linha.is_empty() || conteudo_linha[0] == "c" {
            continue;
        }

        if conteudo_linha[0] == "p" {
            let num_atomos: usize = conteudo_linha[2].parse().expect(&format!(
                "Erro ao converter {} para i32 (linha: {})",
                conteudo_linha[2], indice_linha
            ));

            let _ : usize = conteudo_linha[3].parse().expect(&format!(
                "Erro ao converter {} para i32 (linha: {})",
                conteudo_linha[2], indice_linha
            ));

            atomos = (1..=num_atomos as u32).map(|_| Atomo {
                valor: None,
            }).collect();

            tem_p = true;
            continue;
        }

        if !tem_p {
            eprintln!("Formato CNF invalido, linha p nao encontrada");
            std::process::exit(1);
        }

        let literais: HashSet<i32> = conteudo_linha
            .iter()
            .take_while(|s| **s != "0")
            .map(|s| {
                s.parse::<i32>()
                    .expect(&format!("Erro ao converter '{}' para i32 (linha: {})", s, indice_linha))
            })
            .collect();

        clausulas.push(Clausula {
            literais
        });

        indice_linha += 1;
    }

    // Chamada para o DPLL
    let sat = dpll(clausulas, &mut atomos);

    // Escreve a saida
    let mut file = File::create(format!("{}.res", nome_arquivo)).expect("Erro ao criar o arquivo result.res");
    let error_msg = "Erro ao escrever no arquivo";

    if sat {
        writeln!(file, "SAT").expect(error_msg);
        for (index, atomo) in atomos.iter().enumerate() {
            match atomo.valor {
                Some(valor) => {
                    let literal = if valor { index as i32 + 1 } else { -(index as i32 + 1) };
                    write!(file, "{} ", literal).expect(error_msg);
                }
                None => write!(file, "{} ", index + 1).expect(error_msg),
            }
        }
        writeln!(file, "0").expect(error_msg);
    } else {
        writeln!(file, "UNSAT").expect(error_msg);
    }
}

fn dpll(mut clausulas: Vec<Clausula>, valoracao: &mut Vec<Atomo>) -> bool {
    simplifica(&mut clausulas, valoracao);

    if clausulas.is_empty() {
        return true;
    }

    if clausulas.iter().any(|clausula| clausula.literais.is_empty()) {
        return false;
    }

    let literal = escolher_literal(&clausulas);

    let mut literais1 = HashSet::new();
    literais1.insert(literal);

    clausulas.push(Clausula { literais: literais1 });

    if dpll(clausulas.clone(), valoracao) {
        return true;
    }

    let mut literais2 = HashSet::new();
    literais2.insert(-literal);

    clausulas.pop();
    clausulas.push(Clausula { literais: literais2 });

    if dpll(clausulas, valoracao) {
        return true;
    }

    false
}

fn simplifica(clausulas: &mut Vec<Clausula>, valoracao: &mut Vec<Atomo>) {
    loop {
        let clausula: Option<i32> = clausulas
            .iter()
            .find(|clausula| clausula.literais.len() == 1)
            .and_then(|clausula| clausula.literais.iter().next().copied());

            match clausula {
                Some(literal) => {
                    clausulas.retain(|clausula| !clausula.literais.contains(&literal));
                    clausulas.iter_mut().for_each(|clausula| {
                        clausula.literais.remove(&-literal);
                    });
                    valoracao[(literal.abs() - 1) as usize].valor = if literal < 0 {Some(false)} else {Some(true)};
                },
                None => return,
            }
    }
}

// Heuristica MOM
fn escolher_literal(clausulas: &Vec<Clausula>) -> i32 {
    // Encontra o tamanho mínimo das cláusulas
    let min_len = clausulas
        .iter()
        .map(|clausula| clausula.literais.len())
        .min()
        .unwrap();

    let mut frequencias: HashMap<i32, usize> = HashMap::new();

    // Conta a frequência dos literais nas cláusulas de tamanho mínimo
    clausulas
        .iter()
        .filter(|clausula| clausula.literais.len() == min_len)
        .for_each(|clausula| {
            clausula.literais.iter().for_each(|&literal| {
                *frequencias.entry(literal).or_insert(0) += 1;
            });
        });

    // Encontra o literal com a maior frequência (primeira ocorrencia em caso de empate)
    frequencias.into_iter().max_by_key(|&(_, count)| count).map(|(literal, _)| literal).unwrap()
}