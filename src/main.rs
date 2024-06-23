/*
 * main.rs
 * -------------------------
 * Author  : Kieran van Gelder
 * Id      : 14033623
 * Date    : 2024
 * Version : 0.1
 * -------------------------
 * The main file, used as a import test and showcase.
 * lib.rs is the "main" file that collects and reexports the other files.
 * also houses some plotting functions to plot some data about the e-graphs
 * for the rest this file can be safely ignored
 * 
 * Some rustic things that might be usefull to know:
 * - use /// to indicate that a comment should show up in the function tooltip
 * - 'return n;' can be rewritten as just 'n', though
 *          for clarity sake this isnt used in this library
 * - #[derive(...)] auto generates certain functionality for structs and enums
 * - in lib.rs are some test functions, you can run these in vsc or from the terminal
 * 
 */

use egs;
use egs::{Sexp, egraph::EGraph, parser};

//a carbon copy of the extract_example function in the tests under lib.rs
fn main() {
    static PATH: &str = "src/testsuite/";

    let filepath = &format!("{PATH}ints/example.txt");
    let rulepath = &format!("src/rulesets/rulesetA.txt");
    let iterations = 3;
    rewrite_extract(filepath, rulepath, iterations);
}

/// convert file into a egraph, rewrite it n times according to the rulefile
/// a extract the best candidate using extract_logical
pub fn rewrite_extract(filepath: &str, rulepath: &str, n: u32){
    let sexp: Sexp = parser::parse_file(filepath).unwrap();
    let mut g = EGraph::new();
    let root_id = g.insert_sexpr(sexp);
    let ruleset = &egs::read_ruleset(rulepath);

    for i in 0..n{
        print!("rewrite {}\n", i);
        g.rewrite_ruleset(ruleset);
    }
    print!("\n\n");

    g.print();
    

    if let Some(str) =  g.extract_logical(root_id){
        if let Ok(res) = parser::parse_str(&str){
            egs::pretty_print(&res, 10);
        }
    } else{
        print!("\nFailure to find extractable sexpr\n");
    }
}






//plotting some data, didn't really fit with the lib.rs tests
#[cfg(test)]
mod tests {
    use super::*; //allows this module to use previous scope
    use egs::EGraph;
    use egs::pattern::read_ruleset;
    extern crate plotters;
    use plotters::prelude::*;

    use std::time::Instant;

    static PATH: &str = "src/test terms/";

    /// to test rewriting a graph multiple times
    #[test]
    pub fn egraph_mass_rewrite() {
        let s = Instant::now();
        let filepath = format!("{PATH}ints/example.txt");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        let root_id = g.insert_sexpr(sexp);

        //data
        let mut uf_size = Vec::<(i32, i32)>::new();
        let mut n_enodes = Vec::<(i32, i32)>::new();
        let mut n_classes = Vec::<(i32, i32)>::new();
        let mut edits = Vec::<(i32, i32)>::new();

        let ruleset = &read_ruleset(&format!("src/rulesets/rulesetA.txt"));

        //run rewrite saturation
        uf_size.push((0, g.uf_len().try_into().unwrap()));
        n_enodes.push((0, g.n_enodes().try_into().unwrap()));
        n_classes.push((0, g.n_eclasses().try_into().unwrap()));
        edits.push((0,0));
        for i in 1..9{
            let start = Instant::now();
            let edits1 = g.rewrite_ruleset(ruleset);
            let duration = start.elapsed();
            print!("t: {:?}\n", duration);

            uf_size.push((i, g.uf_len().try_into().unwrap()));
            n_enodes.push((i, g.n_enodes().try_into().unwrap()));
            n_classes.push((i, g.n_eclasses().try_into().unwrap()));
            edits.push((i, edits1));
        }

        if let Some(str) =  g.extract_logical(root_id){
            if let Ok(res) = parser::parse_str(&str){
                egs::pretty_print(&res, 10);
            }
        } else{
            print!("\nFailure to find extractable sexpr\n");
        }

        print!("congruence {:?}\n", g.is_congruent());
        print!("canonical  {:?}\n", g.is_canonical());

        let duration = s.elapsed();
        print!("total (not including graphing time): {:?}\n", duration);

        //plotting data stuff
        let mut edits_vec = Vec::<Vec<(i32, i32)>>::new();
        edits_vec.push(edits);
        let mut uf_size_vec = Vec::<Vec<(i32, i32)>>::new();
        uf_size_vec.push(uf_size);
        uf_size_vec.push(n_enodes);
        let mut eclasses_vec = Vec::<Vec<(i32, i32)>>::new();
        eclasses_vec.push(n_classes);

        let empty = Vec::<&str>::new();
        let mut legend = Vec::<&str>::new();
        legend.push("unionfind size");
        legend.push("number of enodes");

        plot(edits_vec, "edits", "Δ number of edits", empty.clone());
        plot(eclasses_vec, "eclasses", "number of eclasses", empty.clone());
        
        plot(uf_size_vec, "uf and enodes", "amount", legend);
    }


    #[test]
    //in theory associativity does not terminate, this test is to try and forcibly trigger this condition
    //so a "success" is a e-graph that never stops growing.
    //
    //if you run this test you will find that the test "fails", the egraph does not keep growing.
    //why is this the case? i'm not actually entirely sure. i know that the extract functiona voids repeating seen eclasses
    //but match_pattern needs no such safeguard.
    //Most likely it's because the rewrite options aren't exhaustively explored by the matching algorithm
    fn infinite_saturation_attempt(){
        let filepath = format!("{PATH}ints/mult_by_zero.txt");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        g.insert_sexpr(sexp);

        //data
        let mut uf_size = Vec::<i32>::new();

        //ruleset can either be infinityA or infinityB
        let ruleset = &read_ruleset(&format!("src/rulesets/infinityA.txt"));

        //run rewrite saturation
        uf_size.push(g.uf_len().try_into().unwrap());
        // n_enodes.push((0, g.n_enodes().try_into().unwrap()));
        // n_classes.push((0, g.n_eclasses().try_into().unwrap()));
        // edits.push((0,0));
        for _ in 1..9{
            g.rewrite_ruleset(ruleset);
            uf_size.push(g.uf_len().try_into().unwrap());
        }
        g.print();
        print!("{:?}", uf_size);
    }




    fn plot(data_vectors: Vec<Vec<(i32, i32)>>, name: &str, xaxis: &str, legend: Vec<&str>) {
        // Create a drawing backend with a size of 640x480 pixels
        let filename = format!("{}.png", name);
        let root_area = BitMapBackend::new(&filename, (640, 480))
          .into_drawing_area();
        let w: i32 = data_vectors[0].len().try_into().unwrap();
        let h = data_vectors.iter().flat_map(|vec| vec.iter())
          .map(|&(_, y)| y).max().unwrap_or(0);
        let h = h+5-(h%5);

        // Fill the background with white color
        root_area.fill(&WHITE).unwrap();

        // Create a chart context
        let mut chart = ChartBuilder::on(&root_area)
          .margin(10)
          .x_label_area_size(45)
          .y_label_area_size(45)
          .build_cartesian_2d(0..(w-1), 0..h)
          .unwrap();

        // Configure the mesh (grid lines, tick marks, etc.) and set axis labels
        chart.configure_mesh()
          .x_desc("Number of applied rewrites")
          .y_desc(xaxis)
          .x_label_style(("sans-serif", 18).into_font())
          .y_label_style(("sans-serif", 18).into_font())
          .draw()
          .unwrap();


        
        // Plot each data vector
        for (i, data) in data_vectors.iter().enumerate() {
            // Colors to use for the different series
            let colors = vec![&RED, &BLUE, &GREEN];
            if legend.len() == 0{
                chart.draw_series(LineSeries::new(
                    data.clone(),
                    colors[i % colors.len()],
                )).unwrap();
            }
            else {
            chart.draw_series(LineSeries::new(
                data.clone(),
                colors[i % colors.len()],
            )).unwrap()
            .label(format!("{}", legend[i] ))
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], colors[i % colors.len()]));
            }
        }

        if legend.len() != 0{
            // Configure the legend
            chart.configure_series_labels()
              .background_style(&WHITE.mix(0.8))
              .border_style(&BLACK)
              .draw()
              .unwrap();
        }

    // Save the result to a file
    root_area.present().unwrap();
    }
}