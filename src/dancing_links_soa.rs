// DL = Dancing Links
// SoA = implemented using Struct-of-Arrays approach

use itertools::Itertools;
use prettytable::{Cell, Row, Table};
use std::collections::HashMap;
use std::io::prelude::*;

use crate::word_reprs::build_word_representations;

enum Dir {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug)]
pub struct DLMatrix {
    right: Vec<u16>,
    left: Vec<u16>,
    up: Vec<u16>,
    down: Vec<u16>,
    column: Vec<u16>,
    y: Vec<i16>, // used as size if the node is column header, i.e. when the value is < 0
    columns: HashMap<u16, u16>, // column node for given x
    reverse_columns: HashMap<u16, u16>, // column x for given node pointer
    rows: HashMap<u16, u16>, // first cell for given y
}

impl DLMatrix {
    pub fn new() -> Self {
        DLMatrix {
            columns: HashMap::new(),
            reverse_columns: HashMap::new(),
            rows: HashMap::new(),
            right: vec![0],
            left: vec![0],
            up: vec![0],
            down: vec![0],
            column: vec![0],
            y: vec![0],
        }
    }

    fn set(&mut self, src: u16, direction: Dir, dst: u16) {
        match direction {
            Dir::Up => self.up[src as usize] = dst,
            Dir::Right => self.right[src as usize] = dst,
            Dir::Down => self.down[src as usize] = dst,
            Dir::Left => self.left[src as usize] = dst,
        }
    }

    #[inline]
    fn get_neigh_ptr(&self, ptr: u16, direction: Dir) -> u16 {
        match direction {
            Dir::Up => self.up[ptr as usize],
            Dir::Right => self.right[ptr as usize],
            Dir::Down => self.down[ptr as usize],
            Dir::Left => self.left[ptr as usize],
        }
    }

    #[inline]
    fn get_column_ptr(&self, ptr: u16) -> u16 {
        self.column[ptr as usize]
    }

    #[inline(always)]
    fn root_ptr(&self) -> u16 {
        0
    }

    // new_node_factory(ptr) must return a DLNode struct that has valid pointers
    fn add_node<F>(&mut self, f: F) -> u16
    where
        F: Fn(u16) -> (u16, u16, u16, u16, u16, i16),
    {
        let ptr = self.y.len() as u16;
        let (left, up, right, down, column, y) = f(ptr);
        self.left.push(left);
        self.up.push(up);
        self.right.push(right);
        self.down.push(down);
        self.y.push(y);
        self.column.push(column);

        self.set(self.get_neigh_ptr(ptr, Dir::Left), Dir::Right, ptr);
        self.set(self.get_neigh_ptr(ptr, Dir::Up), Dir::Down, ptr);
        self.set(self.get_neigh_ptr(ptr, Dir::Right), Dir::Left, ptr);
        self.set(self.get_neigh_ptr(ptr, Dir::Down), Dir::Up, ptr);

        ptr
    }

    fn add_column(&mut self, x: u16) -> u16 {
        if self.columns.contains_key(&x) {
            return *self.columns.get(&x).unwrap();
        }
        let (root_left_ptr, root_ptr) = (
            self.get_neigh_ptr(self.root_ptr(), Dir::Left),
            self.root_ptr(),
        );
        let ptr = self.add_node(|ptr| (root_left_ptr, ptr, root_ptr, ptr, ptr, -1));
        self.columns.insert(x, ptr);
        self.reverse_columns.insert(ptr, x);
        ptr
    }

    fn add_cell(&mut self, x: u16, y: u16) -> u16 {
        let col_ptr = if !self.columns.contains_key(&x) {
            self.add_column(x)
        } else {
            *self.columns.get(&x).unwrap()
        };
        self.y[col_ptr as usize] -= 1; // Increase size by one; TODO: separate to a different function
        let col_up_ptr = self.get_neigh_ptr(col_ptr, Dir::Up);

        let row_ptrs = if self.rows.contains_key(&y) {
            let row_start_ptr = *self.rows.get(&y).unwrap();
            let row_end_ptr = self.get_neigh_ptr(row_start_ptr, Dir::Left);
            (Some(row_start_ptr), Some(row_end_ptr))
        } else {
            (None, None)
        };

        let ptr = self.add_node(|ptr| {
            (
                row_ptrs.1.unwrap_or(ptr),
                col_up_ptr,
                row_ptrs.0.unwrap_or(ptr),
                col_ptr,
                col_ptr,
                y.try_into().unwrap(),
            )
        });

        if row_ptrs == (None, None) {
            self.rows.insert(y, ptr);
        }

        ptr
    }

    /// Prefer from_bool_rows as more performant
    fn from_bool_columns(columns: &Vec<Vec<bool>>) -> Self {
        let mut res = Self::new();
        for (x, column) in columns.iter().enumerate() {
            for (y, value) in column.iter().enumerate() {
                if *value {
                    res.add_cell(x as u16, y as u16);
                }
            }
        }
        res
    }

    fn from_bool_rows(rows: &Vec<Vec<bool>>) -> Self {
        let mut res = Self::new();
        for (y, row) in rows.iter().enumerate() {
            for (x, value) in row.iter().enumerate() {
                if *value {
                    res.add_cell(x as u16, y as u16);
                }
            }
        }
        res
    }

    #[cfg(not(debug_assertions))]
    fn sanity_check(&self) {}

    #[cfg(debug_assertions)]
    fn sanity_check(&self) {
        /*    let r = self.root_ptr();
        let mut c = r;
        loop {
            c = self.get_neigh_ptr(c, Dir::Right).unwrap();
            if c == r {
                break;
            }
            self.node_sanity_check(c);
            self.column_sanity_check(c);
            let mut j = c;
            loop {
                j = self.get_neigh_ptr(j, Dir::Down).unwrap();
                if j == c {
                    break;
                }
                self.node_sanity_check(j);
            }
        }*/
    }
    /*
        fn node_sanity_check(&self, ptr: u16) {
            if self
                .get_neigh_ptr(self.get_neigh_ptr(ptr, Dir::Left).unwrap(), Dir::Right)
                .unwrap()
                != ptr
            {
                dbg!(&ptr);
                panic!();
            }
            if self
                .get_neigh_ptr(self.get_neigh_ptr(ptr, Dir::Up).unwrap(), Dir::Down)
                .unwrap()
                != ptr
            {
                panic!();
            }
            if self
                .get_neigh_ptr(self.get_neigh_ptr(ptr, Dir::Right).unwrap(), Dir::Left)
                .unwrap()
                != ptr
            {
                panic!();
            }
            if self
                .get_neigh_ptr(self.get_neigh_ptr(ptr, Dir::Down).unwrap(), Dir::Up)
                .unwrap()
                != ptr
            {
                panic!();
            }
        }

        fn column_sanity_check(&self, col_ptr: u16) {
            let mut j = col_ptr;
            let mut ctr = 0;
            loop {
                j = self.get_neigh_ptr(j, Dir::Down).unwrap();
                if j == col_ptr {
                    break;
                }
                ctr += 1;
            }
            match self.get_node(col_ptr).unwrap() {
                DLNode::Column { nav: _, size, x: _ } => {
                    if *size != ctr {
                        dbg!(col_ptr);
                        dbg!(ctr);
                        panic!();
                    }
                }
                _ => panic!(),
            };
        }
    */

    #[inline]
    fn unlink_left_right(&mut self, ptr: u16) {
        self.sanity_check();
        let left = self.get_neigh_ptr(ptr, Dir::Left);
        let right = self.get_neigh_ptr(ptr, Dir::Right);
        self.set(right, Dir::Left, left);
        self.set(left, Dir::Right, right);
    }

    #[inline]
    fn relink_left_right(&mut self, ptr: u16) {
        self.sanity_check();
        let left = self.get_neigh_ptr(ptr, Dir::Left);
        let right = self.get_neigh_ptr(ptr, Dir::Right);
        self.set(right, Dir::Left, ptr);
        self.set(left, Dir::Right, ptr);
    }

    #[inline]
    fn unlink_up_down(&mut self, ptr: u16) {
        self.sanity_check();
        let up = self.get_neigh_ptr(ptr, Dir::Up);
        let down = self.get_neigh_ptr(ptr, Dir::Down);
        self.set(down, Dir::Up, up);
        self.set(up, Dir::Down, down);
        let col = self.get_column_ptr(ptr);
        if col != ptr {
            self.y[ptr as usize] += 1; // Decrease size by one. Todo: separate function.
        }
    }

    #[inline]
    fn relink_up_down(&mut self, ptr: u16) {
        self.sanity_check();
        let up = self.get_neigh_ptr(ptr, Dir::Up);
        let down = self.get_neigh_ptr(ptr, Dir::Down);
        self.set(down, Dir::Up, ptr);
        self.set(up, Dir::Down, ptr);
        let col = self.get_column_ptr(ptr);
        if col != ptr {
            self.y[ptr as usize] -= 1; // Increase size by one. Todo: separate function.
        }
    }

    // Solution = set of columns' x coordinates
    pub fn exact_cover(&mut self) -> Vec<Vec<u16>> {
        let mut o_vals: Vec<u16> = Vec::new();
        let mut solutions: Vec<Vec<u16>> = Vec::new();
        self.exact_cover_rec(0, &mut o_vals, &mut solutions);
        solutions
    }

    fn exact_cover_rec(
        &mut self,
        k: u16,
        partial_solution: &mut Vec<u16>,
        solutions: &mut Vec<Vec<u16>>,
    ) {
        // If the matrix A has no columns, the current partial solution is a valid solution; terminate successfully.
        if self.get_neigh_ptr(self.root_ptr(), Dir::Right) == self.root_ptr() {
            solutions.push(self.current_solution(partial_solution));
            return;
        }

        let c: u16 = self.choose_column();

        // Try every row r that itersects the column c: (this can be parallelized if we clone the matrix)
        let mut r = c;
        loop {
            r = self.get_neigh_ptr(r, Dir::Down);

            if r == c {
                break;
            }

            // Include row r in the partial solution.
            partial_solution.push(r);

            // Every column that is handled by row r is no longer in the equation.
            // Remove all such columns AND all rows that also intersect such columns.
            // We say: cover all such columns.
            let mut j = r;
            loop {
                self.cover(self.get_column_ptr(j));
                j = self.get_neigh_ptr(j, Dir::Right);
                if j == r {
                    break;
                }
            }

            self.exact_cover_rec(k + 1, partial_solution, solutions);

            // Undo covering the columns
            loop {
                j = self.get_neigh_ptr(j, Dir::Left);
                self.uncover(self.get_column_ptr(j));
                if j == r {
                    break;
                }
            }

            partial_solution.pop();
            self.sanity_check();
        }
    }

    fn choose_column(&self) -> u16 {
        let mut s = i16::MAX;
        let mut j = self.root_ptr();
        let mut c = j;
        loop {
            j = self.get_neigh_ptr(j, Dir::Right);
            if j == self.root_ptr() {
                break;
            }
            let size = -self.y[j as usize] - 1;
            if size < s {
                s = size;
                c = j;
            }
        }
        c
    }

    // Cover the column: delete it and all rows that intersect it.
    fn cover(&mut self, col_ptr: u16) {
        self.unlink_left_right(col_ptr);
        let mut row_ptr = self.get_neigh_ptr(col_ptr, Dir::Down);
        while row_ptr != col_ptr {
            let mut j = self.get_neigh_ptr(row_ptr, Dir::Right);
            while j != row_ptr {
                self.unlink_up_down(j);
                j = self.get_neigh_ptr(j, Dir::Right);
            }
            row_ptr = self.get_neigh_ptr(row_ptr, Dir::Down);
        }
    }

    // Uncover the column: undelete it and all rows that intersect it.
    fn uncover(&mut self, col_ptr: u16) {
        let mut row_ptr = self.get_neigh_ptr(col_ptr, Dir::Up);
        while row_ptr != col_ptr {
            let mut j = self.get_neigh_ptr(row_ptr, Dir::Left);
            while j != row_ptr {
                self.relink_up_down(j);
                j = self.get_neigh_ptr(j, Dir::Left);
            }
            row_ptr = self.get_neigh_ptr(row_ptr, Dir::Up);
        }

        self.relink_left_right(col_ptr);
    }

    fn current_solution(&mut self, partial_solution: &mut Vec<u16>) -> Vec<u16> {
        let mut res: Vec<u16> = Vec::new();
        for &ptr in partial_solution.iter() {
            res.push(self.y[ptr as usize] as u16);
        }
        res
    }

    fn print(&self) {
        let root_ptr = self.root_ptr();
        let mut columns = HashMap::new();
        let mut cells = HashMap::new();
        let mut col_ptr = root_ptr;
        let mut max_x = 0;
        let mut max_y = 0;
        loop {
            col_ptr = self.get_neigh_ptr(col_ptr, Dir::Right);
            if col_ptr == root_ptr {
                break;
            }
            let x = *self.reverse_columns.get(&col_ptr).unwrap();
            columns.insert(x, col_ptr);
            if x > max_x {
                max_x = x;
            }

            let mut ptr = col_ptr;
            loop {
                ptr = self.get_neigh_ptr(ptr, Dir::Down);
                if ptr == col_ptr {
                    break;
                }
                let y = self.y[ptr as usize];
                cells.insert((x, y), ptr);
                if y > max_y {
                    max_y = y;
                }
            }
        }

        let mut lines = Vec::new();
        let mut column_names = Vec::new();
        column_names.push(String::from(""));
        for column_no in 0..=max_x {
            column_names.push(format!("{}", column_no));
        }
        lines.push(column_names);

        let mut column_ptrs = vec![String::from("C")];
        for column_no in 0..=max_x {
            column_ptrs.push(if let Some(ptr) = columns.get(&column_no) {
                format!("{}", ptr)
            } else {
                String::from("")
            });
        }
        lines.push(column_ptrs);

        for line_no in 0..=max_y {
            let mut line = vec![format!("{}", line_no)];
            for column_no in 0..=max_x {
                let index = (column_no, line_no);
                line.push(if let Some(ptr) = cells.get(&index) {
                    format!("{}", ptr)
                } else {
                    String::from("")
                });
            }
            lines.push(line);
        }

        let mut table = Table::new();
        for line in lines {
            table.add_row(Row::new(
                line.iter().map(|s| Cell::new(s.as_str())).collect(),
            ));
        }
        // Print the table to stdout
        table.printstd();
    }
}

#[test]
fn test_create_dlmatrix() {
    let columns = vec![
        vec![true, false, true],
        vec![false, true, false],
        vec![true, true, true],
    ];
    let m = DLMatrix::from_bool_columns(&columns);
    dbg!(m);
    // TODO: add a way to traverse the matrix
}

#[test]
fn test_simple_exact_cover() {
    let columns = vec![
        vec![true, false, true],
        vec![true, true, false],
        vec![false, true, false],
    ];
    let mut m = DLMatrix::from_bool_columns(&columns);
    dbg!(&m);
    let solutions = m.exact_cover();
    println!("Solutions size: {}", solutions.len());
    assert_eq!(solutions.len(), 1);
    for (i, solution) in solutions.iter().enumerate() {
        println!("Solution {}", i);
        dbg!(&solution);
        println!("");
    }
}

#[test]
fn test_simple_exact_cover_3el_col() {
    let columns = vec![
        vec![true, true, true],
        vec![false, true, false],
        vec![false, true, true],
    ];
    let mut m = DLMatrix::from_bool_columns(&columns);
    dbg!(&m);
    let solutions = m.exact_cover();
    println!("Solutions size: {}", solutions.len());
    assert_eq!(solutions.len(), 1);
    for (i, solution) in solutions.iter().enumerate() {
        println!("Solution {}", i);
        dbg!(&solution);
        println!("");
    }
}

#[test]
fn test_simple_exact_cover_multi_solution() {
    let columns = vec![
        vec![true, false, true],
        vec![true, false, true],
        vec![false, true, true],
    ];
    let mut m = DLMatrix::from_bool_columns(&columns);
    dbg!(&m);
    let solutions = m.exact_cover();
    println!("Solutions size: {}", solutions.len());
    assert_eq!(solutions.len(), 2);
    for (i, solution) in solutions.iter().enumerate() {
        println!("Solution {}", i);
        dbg!(&solution);
        println!("");
    }
}

#[test]
fn test_medium_exact_cover() {
    let columns = vec![
        vec![false, true, false, true, false, false],
        vec![false, false, true, false, true, false],
        vec![true, false, true, false, false, false],
        vec![false, true, false, true, false, true],
        vec![true, false, false, false, false, true],
        vec![true, false, true, false, false, false],
        vec![false, true, false, false, false, true],
    ];
    let mut m = DLMatrix::from_bool_columns(&columns);

    dbg!(&m);
    let solutions = m.exact_cover();
    println!("Solutions size: {}", solutions.len());
    assert_eq!(solutions.len(), 1);
    for (i, solution) in solutions.iter().enumerate() {
        println!("Solution {}", i);
        for row in solution.iter() {
            print!("{}", *row);
        }
        println!("");
    }
}

#[test]
fn test_weird_exact_cover() {
    let columns = vec![
        vec![false],
        vec![false],
        vec![false],
        vec![false],
        vec![false],
        vec![false],
        vec![false],
        vec![true],
        vec![false],
        vec![false],
        vec![false],
        vec![false],
        vec![true],
        vec![false],
        vec![false],
        vec![false],
        vec![false],
        vec![true],
        vec![false],
        vec![true],
        vec![false],
        vec![false],
        vec![false],
        vec![false],
        vec![true],
    ];
    let mut m = DLMatrix::from_bool_columns(&columns);
    dbg!(&m);
    m.print();
    let solutions = m.exact_cover();
    println!("Solutions size: {}", solutions.len());
}

fn transpose<T>(v: Vec<Vec<T>>) -> Vec<Vec<T>> {
    assert!(!v.is_empty());
    let len = v[0].len();
    let mut iters: Vec<_> = v.into_iter().map(|n| n.into_iter()).collect();
    (0..len)
        .map(|_| {
            iters
                .iter_mut()
                .map(|n| n.next().unwrap())
                .collect::<Vec<T>>()
        })
        .collect()
}

pub fn dlx_words(words: Vec<String>) {
    let (repr_map, reprs) = build_word_representations(&words);

    let mut rows: Vec<Vec<bool>> = reprs
        .iter()
        .map(|w| (0..26).map(|b| (w & (1 << b)) != 0).collect())
        .collect();

    for row in rows.iter_mut() {
        row.push(false); // Column for 1-letter long dummy
                         // We need a dummy because we need to cover all 26 letters
                         // The last column of the matrix will represent "is this row a dummy". We need exactly one dummy row in each solution.
    }
    for dummy_letter in b'a'..=b'z' {
        let mut dummy_row: Vec<bool> = (b'a'..=b'z').map(|b| b == dummy_letter).collect();
        dummy_row.push(true);
        rows.push(dummy_row);
    }

    // Construct the matrix and run exact cover
    let mut dlm = DLMatrix::from_bool_rows(&rows);
    let solutions = dlm.exact_cover();
    let mut ctr = 0;
    for solution in solutions.iter() {
        let repr_combo: Vec<&u32> = solution
            .iter()
            .filter_map(|index| reprs.get(*index as usize))
            .collect();
        ctr += combo_pretty_print(repr_combo, &repr_map);
    }
    println!("Solutions count: {}", ctr);
}

/// Returns number of combos printed
fn combo_pretty_print(combo: Vec<&u32>, reprs: &HashMap<u32, Vec<&str>>) -> usize {
    let mut ctr = 0;
    for word_combo in combo
        .iter()
        .map(|key| reprs.get(key).unwrap())
        .multi_cartesian_product()
    {
        let word_combo = word_combo.into_iter().fold(String::new(), |mut acc, word| {
            acc.push_str(word);
            acc.push_str(" ");
            acc
        });
        let _ = writeln!(std::io::stdout(), "{}", word_combo);
        ctr += 1;
    }
    ctr
}
