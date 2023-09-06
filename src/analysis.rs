use crate::asm::Reg;
use crate::graph::Graph;
use crate::syntax::{ImmExp, Prim1, SeqExp};
use std::collections::{HashMap, HashSet};
use std::process::ExitStatus;

/* A location where a local variable is stored */
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum VarLocation {
    Reg(Reg),
    Spill(i32), // a (negative) offset to RBP
}

fn liveness_imm(imm: &ImmExp, params: &HashSet<String>) -> HashSet<String> {
    match imm {
        ImmExp::Num(_) | ImmExp::Bool(_) => HashSet::new(),
        ImmExp::Var(x) => {
            if params.contains(x) {
                HashSet::new()
            } else {
                HashSet::from([(*x).clone()])
            }
        }
    }
}

pub fn liveness<Ann>(
    e: &SeqExp<Ann>,
    params: &HashSet<String>,
    live_out: HashSet<String>,
) -> SeqExp<HashSet<String>> {
    match e {
        SeqExp::Imm(imm, _) => SeqExp::Imm((*imm).clone(), &live_out | &liveness_imm(imm, params)),
        SeqExp::Prim1(prim, imm, _) => SeqExp::Prim1(
            *prim,
            (*imm).clone(),
            &live_out | &liveness_imm(imm, params),
        ),
        SeqExp::Prim2(prim, imm1, imm2, _) => SeqExp::Prim2(
            *prim,
            (*imm1).clone(),
            (*imm2).clone(),
            &live_out | &(&liveness_imm(imm1, params) | &liveness_imm(imm2, params)),
        ),
        SeqExp::Array(array, _) => SeqExp::Array(
            array.clone(),
            array
                .iter()
                .fold(live_out, |acc, imm| &acc | &liveness_imm(imm, params)),
        ),
        SeqExp::ArraySet {
            array,
            index,
            new_value,
            ann: _,
        } => {
            let mut ann: HashSet<String> = live_out;
            ann = &ann | &liveness_imm(array, params);
            ann = &ann | &liveness_imm(index, params);
            ann = &ann | &liveness_imm(new_value, params);
            SeqExp::ArraySet {
                array: (*array).clone(),
                index: (*index).clone(),
                new_value: (*new_value).clone(),
                ann: ann,
            }
        }
        SeqExp::MakeClosure {
            arity,
            label,
            env,
            ann: _,
        } => SeqExp::MakeClosure {
            arity: *arity,
            label: label.clone(),
            env: (*env).clone(),
            ann: &live_out | &liveness_imm(env, params),
        },
        SeqExp::CallClosure { fun, args, ann: _ } => SeqExp::CallClosure {
            fun: (*fun).clone(),
            args: args.clone(),
            ann: args
                .iter()
                .fold(&live_out | &liveness_imm(fun, params), |acc, imm| {
                    &acc | &liveness_imm(imm, params)
                }),
        },
        SeqExp::Let {
            var,
            bound_exp,
            body,
            ann: _,
        } => {
            let body_analysis: SeqExp<HashSet<String>> = liveness(body, params, live_out);
            let mut ann: HashSet<String> = body_analysis.ann();
            if ann.contains(var) {
                ann.remove(var);
            }
            let bound_exp_analysis: SeqExp<HashSet<String>> = liveness(bound_exp, params, ann);
            ann = bound_exp_analysis.ann();
            SeqExp::Let {
                var: (*var).clone(),
                bound_exp: Box::new(bound_exp_analysis),
                body: Box::new(body_analysis),
                ann: ann,
            }
        }
        SeqExp::If {
            cond,
            thn,
            els,
            ann: _,
        } => {
            let thn_analysis: SeqExp<HashSet<String>> = liveness(thn, params, live_out.clone());
            let els_analysis: SeqExp<HashSet<String>> = liveness(els, params, live_out.clone());
            let ann: HashSet<String> = &thn_analysis.ann() | &els_analysis.ann();
            SeqExp::If {
                cond: (*cond).clone(),
                thn: Box::new(thn_analysis),
                els: Box::new(els_analysis),
                ann: &ann | &liveness_imm(cond, params),
            }
        }
    }
}

// Check if ImmExp is a non-parameter varaible
fn extract_variable(imm: &ImmExp, live: &HashSet<String>) -> Option<String> {
    match imm {
        ImmExp::Var(x) => {
            if live.contains(x) {
                Some(x.clone())
            } else {
                None
            }
        }
        _ => None,
    }
}

fn deal_conflict(
    live: &HashSet<String>,
    conflict_graph: &mut Graph<String>,
    non_conflict_graph: &mut Graph<String>,
) -> () {
    for var1 in live.iter() {
        for var2 in live.iter() {
            if var1 != var2 {
                if !non_conflict_graph.contains_edge(var1, var2) {
                    conflict_graph.insert_edge(var1.clone(), var2.clone());
                }
            }
        }
    }
}

fn conflicts_helper(
    e: &SeqExp<HashSet<String>>,
    conflict_graph: &mut Graph<String>,
    non_conflict_graph: &mut Graph<String>,
) -> Option<String> {
    match e {
        SeqExp::Imm(imm, ann) => {
            deal_conflict(ann, conflict_graph, non_conflict_graph);
            extract_variable(imm, ann)
        }
        SeqExp::Prim1(prim, imm, ann) => {
            deal_conflict(ann, conflict_graph, non_conflict_graph);
            match prim {
                Prim1::Print => extract_variable(imm, ann),
                _ => None,
            }
        }
        SeqExp::Prim2(.., ann) => {
            deal_conflict(ann, conflict_graph, non_conflict_graph);
            None
        }
        SeqExp::Array(_, ann) => {
            deal_conflict(ann, conflict_graph, non_conflict_graph);
            None
        }
        SeqExp::ArraySet { ann, .. } => {
            deal_conflict(ann, conflict_graph, non_conflict_graph);
            None
        }
        SeqExp::MakeClosure { ann, .. } => {
            deal_conflict(ann, conflict_graph, non_conflict_graph);
            None
        }
        SeqExp::CallClosure { ann, .. } => {
            deal_conflict(ann, conflict_graph, non_conflict_graph);
            None
        }
        SeqExp::Let {
            var,
            bound_exp,
            body,
            ann,
        } => {
            deal_conflict(ann, conflict_graph, non_conflict_graph);
            match conflicts_helper(bound_exp, conflict_graph, non_conflict_graph) {
                Some(x) => non_conflict_graph.insert_edge(var.clone(), x.clone()),
                None => {}
            };
            conflict_graph.insert_vertex(var.clone());
            non_conflict_graph.insert_vertex(var.clone());
            conflicts_helper(body, conflict_graph, non_conflict_graph)
        }
        SeqExp::If {
            cond: _,
            thn,
            els,
            ann,
        } => {
            deal_conflict(ann, conflict_graph, non_conflict_graph);
            let var1: String = match conflicts_helper(thn, conflict_graph, non_conflict_graph) {
                Some(x) => x,
                None => String::new(),
            };
            let var2: String = match conflicts_helper(els, conflict_graph, non_conflict_graph) {
                Some(x) => x,
                None => String::new(),
            };
            if var1 == var2 && !var1.is_empty() {
                Some(var1)
            } else {
                None
            }
        }
    }
}

pub fn conflicts(e: &SeqExp<HashSet<String>>) -> Graph<String> {
    let mut conflict_graph: Graph<String> = Graph::new();
    let mut non_conflict_graph: Graph<String> = Graph::new();
    conflicts_helper(e, &mut conflict_graph, &mut non_conflict_graph);
    conflict_graph
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Status {
    Normal,
    Trouble,
}

pub fn allocate_registers(
    conflicts: Graph<String>,
    registers: &[Reg],
) -> HashMap<String, VarLocation> {
    let mut variable_graph: Graph<String> = conflicts.clone();
    let mut variable_stack: Vec<(String, Status)> = Vec::new();
    // Loop until all variables are pushed into the variable stack
    while variable_graph.num_vertices() != 0 {
        let mut var: String = String::new();
        // Look for a variable with neighbors less than number of registers
        for variable in variable_graph.vertices().iter() {
            match variable_graph.neighbors(variable) {
                Some(neighbors) => {
                    if neighbors.len() < registers.len() {
                        var = variable.clone();
                        break;
                    }
                }
                None => {
                    panic!("Variable is guaranteed to be in the conflict graph")
                }
            };
        }
        if var.is_empty() {
            // If not found, look for a variable with greatest degree
            let mut max_num: usize = 0;
            let mut max_var: String = String::new();
            for variable in variable_graph.vertices().iter() {
                let num: usize = match variable_graph.neighbors(variable) {
                    Some(neighbors) => neighbors.len(),
                    None => 0,
                };
                if num >= max_num {
                    max_num = num;
                    max_var = variable.clone();
                }
            }
            // Remove variable from the conflict graph
            // Push variable into the variable stack
            variable_graph.remove_vertex(&max_var);
            variable_stack.push((max_var, Status::Trouble));
        } else {
            // If found
            // Remove variable from the conflict graph
            // Push variable into the variable stack
            variable_graph.remove_vertex(&var);
            variable_stack.push((var, Status::Normal));
        }
    }
    // Check all variables are removed from the conflict graph
    // Check all varaibles are pushed into the variable stack
    assert!(variable_graph.num_vertices() == 0);
    assert!(variable_stack.len() == conflicts.num_vertices());
    let mut allocation: HashMap<String, VarLocation> = HashMap::new();
    let mut stack_counter: i32 = 0;
    // Allocate registers or stack spaces for all variables
    while !variable_stack.is_empty() {
        // Pop varaible from variable stack
        let (var, status) = match variable_stack.pop() {
            Some((variable, status)) => (variable, status),
            None => panic!("Variable stack is guaranteed not empty"),
        };
        // Insert variable back into the conflict graph
        variable_graph.insert_vertex(var.clone());
        match conflicts.neighbors(&var) {
            Some(neighbors) => {
                for neighbor in neighbors.iter() {
                    match variable_graph.neighbors(neighbor) {
                        Some(_) => {
                            // neighbor in variable graph
                            variable_graph.insert_edge(var.clone(), neighbor.clone());
                        }
                        None => {
                            // neighbor not in variable graph
                        }
                    }
                }
            }
            None => panic!("Variable is guaranteed to be in the conflict graph"),
        };
        // Allocate register for variable
        let mut available_registers: Vec<Reg> = registers.to_vec();
        // Registers of the neighbors should not be allocated for variable
        match variable_graph.neighbors(&var) {
            Some(neighbors) => {
                for neighbor in neighbors.iter() {
                    match allocation.get(neighbor) {
                        Some(VarLocation::Reg(reg)) => {
                            available_registers.retain(|&r| r != *reg);
                        }
                        Some(VarLocation::Spill(_)) => {
                            panic!(
                                "Variables on the stack should be removed from the conflict graph"
                            )
                        }
                        None => {
                            panic!("Variables in the conflict graph should have been allocated already")
                        }
                    }
                }
            }
            None => panic!("Variable is guaranteed to be in the conflict graph"),
        };
        match status {
            // If the variable is normal
            Status::Normal => {
                allocation.insert(var, VarLocation::Reg(available_registers[0]));
            }
            // If the variable is troublesome
            Status::Trouble => {
                if available_registers.len() == 0 {
                    stack_counter += 1;
                    allocation.insert(var, VarLocation::Spill(-8 * stack_counter));
                } else {
                    allocation.insert(var, VarLocation::Reg(available_registers[0]));
                }
            }
        };
    }
    allocation
}
