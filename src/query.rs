use std::fmt::Write;
use tokio_postgres::types::ToSql;

pub type PgArguments = Vec<Box<dyn ToSql + Send + Sync>>;

pub fn pg_pattern_escape(s: &str) -> String {
    s.replace('%', "\\%").replace('_', "\\_")
}

pub struct QueryBuilder {
    table: &'static str,
    selects: Vec<String>,
    wheres: Vec<String>,
    pub order: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
    args: PgArguments,
}

impl QueryBuilder {
    pub fn new(table: &'static str) -> Self {
        Self {
            table,
            selects: Vec::new(),
            wheres: Vec::new(),
            limit: None,
            offset: None,
            order: None,
            args: PgArguments::default(),
        }
    }

    pub fn add_select(&mut self, field: impl Into<String>) -> &mut Self {
        self.selects.push(field.into());
        self
    }

    pub fn add_where_raw(&mut self, clause: impl Into<String>) -> &mut Self {
        self.wheres.push(clause.into());
        self
    }

    pub fn add_where(&mut self, clause: impl Into<String>) -> &mut Self {
        self.wheres.push(clause.into());
        self
    }

    pub fn bind(&mut self, value: impl ToSql + Send + Sync + 'static) -> String {
        // TODO: optimize
        self.args.push(Box::new(value));
        format!("${}", self.args.len())
    }

    pub fn order(&mut self, order: impl Into<String>) -> &mut Self {
        self.order = Some(order.into());
        self
    }

    pub fn limit(&mut self, limit: u32) -> &mut Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(&mut self, offset: u32) -> &mut Self {
        self.offset = Some(offset);
        self
    }

    fn build_stmt(&self) -> String {
        let mut res = "select ".to_owned();
        assert!(!self.selects.is_empty());

        res.push_str(&self.selects[0]);
        for select in &self.selects[1..] {
            res.push(',');
            res.push_str(select);
        }
        res.push_str(" from ");
        res += self.table;
        let mut it = self.wheres.iter();
        if let Some(first) = it.next() {
            res += " where (";
            res += first;
            for rest in it {
                res += ") and (";
                res += rest;
            }
            res.push(')');
        }

        if let Some(order) = &self.order {
            res += " order by ";
            res += order;
        }
        if let Some(limit) = &self.limit {
            write!(res, " limit {limit}").unwrap();
        }
        if let Some(offset) = &self.offset {
            write!(res, " offset {offset}").unwrap();
        }

        res
    }

    pub fn build(self) -> (String, PgArguments) {
        (self.build_stmt(), self.args)
    }
}

pub fn args_to_ref(args: &PgArguments) -> Vec<&(dyn ToSql + Sync)> {
    args.iter()
        .map(|it| it.as_ref() as &(dyn ToSql + Sync))
        .collect()
}
