// DL = Dancing Links

use prettytable::{Cell, Row, Table};

use std::collections::{HashMap, HashSet};

type DLPtr = usize;

enum Dir {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug)]
pub struct DLMatrix {
    arena: Vec<DLNode>,
    columns: HashMap<usize, DLPtr>, // column node for given x
    rows: HashMap<usize, DLPtr>,    // first cell for given y
}

#[derive(Debug, Clone, Copy)]
pub struct DLNav {
    right: DLPtr,
    left: DLPtr,
    up: DLPtr,
    down: DLPtr,
    this: DLPtr,
}

#[derive(Debug, Clone, Copy)]
pub enum DLNode {
    Root {
        nav: DLNav,
    },
    Column {
        nav: DLNav,
        size: usize,
        x: usize,
    },
    Cell {
        nav: DLNav,
        column: DLPtr,
        x: usize,
        y: usize,
    },
}

impl DLNode {
    fn get_nav(&self) -> &DLNav {
        match self {
            DLNode::Root { nav } => nav,
            DLNode::Column { nav, .. } => nav,
            DLNode::Cell { nav, .. } => nav,
        }
    }

    fn get_nav_mut<'a>(&'a mut self) -> &'a mut DLNav {
        match self {
            DLNode::Root { nav } => nav,
            DLNode::Column { nav, .. } => nav,
            DLNode::Cell { nav, .. } => nav,
        }
    }

    fn set(&mut self, direction: Dir, ptr: DLPtr) {
        match direction {
            Dir::Up => self.get_nav_mut().up = ptr,
            Dir::Right => self.get_nav_mut().right = ptr,
            Dir::Down => self.get_nav_mut().down = ptr,
            Dir::Left => self.get_nav_mut().left = ptr,
        }
    }

    fn get(&self, direction: Dir) -> DLPtr {
        match direction {
            Dir::Up => self.get_nav().up,
            Dir::Right => self.get_nav().right,
            Dir::Down => self.get_nav().down,
            Dir::Left => self.get_nav().left,
        }
    }
}

impl DLMatrix {
    pub fn new() -> Self {
        Self::new_with_capacity(256)
    }

    pub fn new_with_capacity(initial_capacity: usize) -> Self {
        let root = DLNode::Root {
            nav: DLNav {
                right: 0,
                left: 0,
                up: 0,
                down: 0,
                this: 0,
            },
        };
        let mut arena = Vec::with_capacity(initial_capacity);
        arena.push(root);
        DLMatrix {
            arena,
            columns: HashMap::new(),
            rows: HashMap::new(),
        }
    }

    fn get_node_mut(&mut self, ptr: DLPtr) -> Option<&mut DLNode> {
        self.arena.get_mut(ptr)
    }

    fn get_node(&self, ptr: DLPtr) -> Option<&DLNode> {
        self.arena.get(ptr)
    }

    fn get_neigh_ptr(&self, ptr: DLPtr, direction: Dir) -> Option<DLPtr> {
        Some(self.get_node(ptr)?.get(direction))
    }

    fn get_neigh_node_mut(&mut self, ptr: DLPtr, direction: Dir) -> Option<&mut DLNode> {
        self.get_node_mut(self.get_neigh_ptr(ptr, direction)?)
    }

    fn get_column_ptr(&self, ptr: DLPtr) -> DLPtr {
        match self.get_node(ptr).unwrap() {
            DLNode::Root { nav: _ } => self.root_ptr(),
            DLNode::Column { nav, .. } => nav.this,
            DLNode::Cell { nav: _, column, .. } => *column,
        }
    }

    fn root(&self) -> &DLNode {
        self.arena.get(0).unwrap()
    }

    fn root_ptr(&self) -> DLPtr {
        0
    }

    // new_node_factory(ptr) must return a DLNode struct that has valid pointers in its nav element
    fn add_node<F>(&mut self, new_node_factory: F) -> DLPtr
    where
        F: Fn(DLPtr) -> DLNode,
    {
        let ptr = self.arena.len();
        self.arena.push(new_node_factory(ptr));
        self.get_neigh_node_mut(ptr, Dir::Left)
            .unwrap()
            .set(Dir::Right, ptr);
        self.get_neigh_node_mut(ptr, Dir::Right)
            .unwrap()
            .set(Dir::Left, ptr);
        self.get_neigh_node_mut(ptr, Dir::Up)
            .unwrap()
            .set(Dir::Down, ptr);
        self.get_neigh_node_mut(ptr, Dir::Down)
            .unwrap()
            .set(Dir::Up, ptr);
        ptr
    }

    fn add_column(&mut self, x: usize) -> DLPtr {
        if self.columns.contains_key(&x) {
            return *self.columns.get(&x).unwrap();
        }
        let (root_left_ptr, root_ptr) = (self.root().get(Dir::Left), self.root_ptr());
        let ptr = self.add_node(|ptr| DLNode::Column {
            nav: DLNav {
                left: root_left_ptr,
                right: root_ptr,
                up: ptr,
                down: ptr,
                this: ptr,
            },
            size: 0,
            x,
        });
        self.columns.insert(x, ptr);
        ptr
    }

    fn add_cell(&mut self, x: usize, y: usize) -> DLPtr {
        let col_ptr = if !self.columns.contains_key(&x) {
            self.add_column(x)
        } else {
            *self.columns.get(&x).unwrap()
        };
        match self.get_node_mut(col_ptr).unwrap() {
            DLNode::Column { nav: _, size, x: _ } => {
                *size += 1;
            }
            _ => unreachable!(),
        };
        let col_up_ptr = self.get_neigh_ptr(col_ptr, Dir::Up).unwrap();

        let row_ptrs = if self.rows.contains_key(&y) {
            let row_start_ptr = *self.rows.get(&y).unwrap();
            let row_end_ptr = self.get_neigh_ptr(row_start_ptr, Dir::Left).unwrap();
            (Some(row_start_ptr), Some(row_end_ptr))
        } else {
            (None, None)
        };

        let ptr = self.add_node(|ptr| DLNode::Cell {
            nav: DLNav {
                left: row_ptrs.1.unwrap_or(ptr),
                right: row_ptrs.0.unwrap_or(ptr),
                up: col_up_ptr,
                down: col_ptr,
                this: ptr,
            },
            column: col_ptr,
            x,
            y,
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
                    res.add_cell(x, y);
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
                    res.add_cell(x, y);
                }
            }
        }
        res
    }

    #[cfg(not(debug_assertions))]
    fn sanity_check(&self) {}

    #[cfg(debug_assertions)]
    fn sanity_check(&self) {
        let r = self.root_ptr();
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
        }
    }

    fn node_sanity_check(&self, ptr: DLPtr) {
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

    fn column_sanity_check(&self, col_ptr: DLPtr) {
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

    fn unlink_left_right(&mut self, ptr: DLPtr) {
        self.sanity_check();
        let left = self.get_neigh_ptr(ptr, Dir::Left).unwrap();
        let right = self.get_neigh_ptr(ptr, Dir::Right).unwrap();
        self.get_node_mut(right).unwrap().set(Dir::Left, left);
        self.get_node_mut(left).unwrap().set(Dir::Right, right);
    }

    fn relink_left_right(&mut self, ptr: DLPtr) {
        self.sanity_check();
        let left = self.get_neigh_ptr(ptr, Dir::Left).unwrap();
        let right = self.get_neigh_ptr(ptr, Dir::Right).unwrap();
        self.get_node_mut(right).unwrap().set(Dir::Left, ptr);
        self.get_node_mut(left).unwrap().set(Dir::Right, ptr);
    }

    fn unlink_up_down(&mut self, ptr: DLPtr) {
        self.sanity_check();
        let up = self.get_neigh_ptr(ptr, Dir::Up).unwrap();
        let down = self.get_neigh_ptr(ptr, Dir::Down).unwrap();
        self.get_node_mut(down).unwrap().set(Dir::Up, up);
        self.get_node_mut(up).unwrap().set(Dir::Down, down);
        let col = self.get_column_ptr(ptr);
        if col != ptr {
            match self.get_node_mut(col).unwrap() {
                DLNode::Column { nav, size, x } => {
                    *size -= 1;
                }
                _ => unreachable!(),
            };
        }
    }

    fn relink_up_down(&mut self, ptr: DLPtr) {
        self.sanity_check();
        let up = self.get_neigh_ptr(ptr, Dir::Up).unwrap();
        let down = self.get_neigh_ptr(ptr, Dir::Down).unwrap();
        self.get_node_mut(down).unwrap().set(Dir::Up, ptr);
        self.get_node_mut(up).unwrap().set(Dir::Down, ptr);
        let col = self.get_column_ptr(ptr);
        if col != ptr {
            match self.get_node_mut(col).unwrap() {
                DLNode::Column { nav, size, x } => {
                    *size += 1;
                }
                _ => unreachable!(),
            };
        }
    }

    // Solution = set of columns' x coordinates
    pub fn exact_cover(&mut self) -> Vec<Vec<usize>> {
        let mut o_vals: Vec<DLPtr> = Vec::new();
        let mut solutions: Vec<Vec<usize>> = Vec::new();
        self.exact_cover_rec(0, &mut o_vals, &mut solutions);
        solutions
    }

    fn exact_cover_rec(
        &mut self,
        k: usize,
        partial_solution: &mut Vec<DLPtr>,
        solutions: &mut Vec<Vec<usize>>,
    ) {
        // If the matrix A has no columns, the current partial solution is a valid solution; terminate successfully.
        if self.get_neigh_ptr(self.root_ptr(), Dir::Right) == Some(self.root_ptr()) {
            solutions.push(self.current_solution(partial_solution));
            return;
        }

        let c: DLPtr = self.choose_column();

        // Try every row r that itersects the column c: (this can be parallelized if we clone the matrix)
        let mut r = c;
        loop {
            r = self.get_neigh_ptr(r, Dir::Down).unwrap();

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
                j = self.get_neigh_ptr(j, Dir::Right).unwrap();
                if j == r {
                    break;
                }
            }

            self.exact_cover_rec(k + 1, partial_solution, solutions);

            // Undo covering the columns
            loop {
                j = self.get_neigh_ptr(j, Dir::Left).unwrap();
                self.uncover(self.get_column_ptr(j));
                if j == r {
                    break;
                }
            }

            partial_solution.pop();
            self.sanity_check();
        }
    }

    fn choose_column(&self) -> DLPtr {
        let mut s = usize::MAX;
        let mut j = self.root_ptr();
        let mut c = j;
        loop {
            j = self.get_neigh_ptr(j, Dir::Right).unwrap();
            if j == self.root_ptr() {
                break;
            }
            match self.get_node(j).unwrap() {
                DLNode::Column { nav: _, size, x: _ } => {
                    if *size < s {
                        s = *size;
                        c = j;
                    }
                }
                _ => unreachable!(),
            };
        }
        c
    }

    // Cover the column: delete it and all rows that intersect it.
    fn cover(&mut self, col_ptr: DLPtr) {
        self.unlink_left_right(col_ptr);
        let mut row_ptr = self.get_neigh_ptr(col_ptr, Dir::Down).unwrap();
        while row_ptr != col_ptr {
            let mut j = self.get_neigh_ptr(row_ptr, Dir::Right).unwrap();
            while j != row_ptr {
                self.unlink_up_down(j);
                j = self.get_neigh_ptr(j, Dir::Right).unwrap();
            }
            row_ptr = self.get_neigh_ptr(row_ptr, Dir::Down).unwrap();
        }
    }

    // Uncover the column: undelete it and all rows that intersect it.
    fn uncover(&mut self, col_ptr: DLPtr) {
        let mut row_ptr = self.get_neigh_ptr(col_ptr, Dir::Up).unwrap();
        while row_ptr != col_ptr {
            let mut j = self.get_neigh_ptr(row_ptr, Dir::Left).unwrap();
            while j != row_ptr {
                self.relink_up_down(j);
                j = self.get_neigh_ptr(j, Dir::Left).unwrap();
            }
            row_ptr = self.get_neigh_ptr(row_ptr, Dir::Up).unwrap();
        }

        self.relink_left_right(col_ptr);
    }

    fn current_solution(&mut self, partial_solution: &mut Vec<DLPtr>) -> Vec<usize> {
        let mut res: Vec<usize> = Vec::new();
        for &ptr in partial_solution.iter() {
            res.push(match self.get_node(ptr).unwrap() {
                DLNode::Cell {
                    nav: _,
                    column: _,
                    x: _,
                    y,
                } => *y,
                _ => unreachable!(),
            });
        }
        res
    }

    fn print(&self) {
        let root_ptr = self.root_ptr();
        let mut columns = HashMap::new();
        let mut cells = HashMap::new();
        let mut col_ptr = root_ptr;
        loop {
            col_ptr = self.get_neigh_ptr(col_ptr, Dir::Right).unwrap();
            if col_ptr == root_ptr {
                break;
            }
            if let DLNode::Column { nav: _, size: _, x } = self.get_node(col_ptr).unwrap() {
                columns.insert(x, col_ptr);
            };
            let mut ptr = col_ptr;
            loop {
                ptr = self.get_neigh_ptr(ptr, Dir::Down).unwrap();
                if ptr == col_ptr {
                    break;
                }
                if let DLNode::Cell {
                    nav: _,
                    column: _,
                    x,
                    y,
                } = self.get_node(ptr).unwrap()
                {
                    cells.insert((*x, *y), ptr);
                }
            }
        }

        let max_x = **columns.keys().max().unwrap_or(&&0);
        let max_y = cells.keys().map(|(_x, y)| *y).max().unwrap_or(0);

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
    let all_columns: Vec<Vec<bool>> = (b'a'..=b'z')
        .map(|b| {
            let ch = b as char;
            words.iter().map(|w| w.contains(ch)).collect::<Vec<bool>>()
        })
        .collect();

    let dummy_row = vec![true; 26 - 1];

    let mut ctr = 0;
    for left_out in 0..26 {
        eprintln!("{}", (left_out + b'a') as char);

        // Remove column corresponding to the left out letter
        let columns: Vec<Vec<bool>> = all_columns
            .clone()
            .into_iter()
            .enumerate()
            .filter(|(i, _)| *i != left_out.into())
            .map(|(_i, c)| c)
            .collect();

        // Remove words containing the left out letter
        let left_out_column: &Vec<bool> = all_columns.get(left_out as usize).unwrap();
        let left_out_rows: HashSet<usize> = left_out_column
            .iter()
            .enumerate()
            .filter(|(_i, b)| **b)
            .map(|(i, _b)| i)
            .collect();
        let mut rows: Vec<Vec<bool>> = transpose(columns)
            .into_iter()
            .enumerate()
            .filter(|(i, _r)| !left_out_rows.contains(i))
            .map(|(_i, r)| r)
            .collect();

        // Ensure every column will be created in the matrix
        rows.push(dummy_row.clone());

        // Construct the matrix and run exact cover
        let mut dlm = DLMatrix::from_bool_rows(&rows);
        let solutions = dlm.exact_cover();
        for solution in solutions {
            if solution.len() == 1 && solution[0] == rows.len() - 1 {
                // This is the dummy row
                continue;
            }
            for index in solution {
                print!("{} ", words[index].as_str());
            }
            println!("");
            ctr += 1;
        }
    }
    dbg!(ctr);
}
