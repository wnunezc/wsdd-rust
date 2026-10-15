use crate::models::project::Project;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProjectColumn {
    #[default]
    Name,
    Domain,
    Php,
}

#[derive(Default)]
pub struct ProjectTable {
    pub filter: String,
    pub column: Option<ProjectColumn>,
    pub descending: bool,
}

impl ProjectTable {
    pub fn sort_by(&mut self, column: ProjectColumn) {
        if self.column == Some(column) {
            self.descending = !self.descending;
        } else {
            self.column = Some(column);
            self.descending = false;
        }
    }

    pub fn visible_projects(&self, projects: &[Project]) -> Vec<Project> {
        let query = self.filter.trim().to_lowercase();
        let mut visible: Vec<_> = projects
            .iter()
            .filter(|p| {
                query.is_empty()
                    || [&p.name, &p.domain, p.php_version.display_name()]
                        .iter()
                        .any(|text| text.to_lowercase().contains(&query))
            })
            .cloned()
            .collect();
        visible.sort_by(|a, b| {
            let order = match self.column.unwrap_or(ProjectColumn::Name) {
                ProjectColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                ProjectColumn::Domain => a.domain.to_lowercase().cmp(&b.domain.to_lowercase()),
                ProjectColumn::Php => version_key(a).cmp(&version_key(b)),
            }
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            .then_with(|| a.name.cmp(&b.name));
            if self.descending {
                order.reverse()
            } else {
                order
            }
        });
        visible
    }
}

fn version_key(project: &Project) -> Vec<u32> {
    project
        .php_version
        .display_name()
        .trim_start_matches("PHP ")
        .split('.')
        .map(|part| {
            part.parse()
                .expect("supported PHP version has numeric components")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::project::PhpVersion;

    fn projects() -> Vec<Project> {
        vec![
            Project::new(
                "Zulu".into(),
                "alpha.dock",
                PhpVersion::Php84,
                String::new(),
            ),
            Project::new(
                "alpha".into(),
                "zulu.dock",
                PhpVersion::Php56,
                String::new(),
            ),
            Project::new("Beta".into(), "beta.dock", PhpVersion::Php74, String::new()),
        ]
    }

    #[test]
    fn toggles_direction_and_resets_on_column_change() {
        let mut table = ProjectTable::default();
        table.sort_by(ProjectColumn::Name);
        assert!(!table.descending);
        table.sort_by(ProjectColumn::Name);
        assert!(table.descending);
        table.sort_by(ProjectColumn::Domain);
        assert!(!table.descending);
    }

    #[test]
    fn sorts_each_column_in_both_directions() {
        for (column, expected) in [
            (ProjectColumn::Name, vec!["alpha", "Beta", "Zulu"]),
            (ProjectColumn::Domain, vec!["Zulu", "Beta", "alpha"]),
            (ProjectColumn::Php, vec!["alpha", "Beta", "Zulu"]),
        ] {
            let mut table = ProjectTable {
                column: Some(column),
                ..Default::default()
            };
            assert_eq!(
                table
                    .visible_projects(&projects())
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>(),
                expected
            );
            table.sort_by(column);
            assert_eq!(
                table
                    .visible_projects(&projects())
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>(),
                expected.into_iter().rev().collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn filters_partial_and_complete_matches_across_all_columns() {
        for (filter, expected) in [
            ("bET", "Beta"),
            ("ZULU.DOCK", "alpha"),
            ("PHP 8.4", "Zulu"),
            ("5.6", "alpha"),
        ] {
            let table = ProjectTable {
                filter: filter.into(),
                ..Default::default()
            };
            let visible = table.visible_projects(&projects());
            assert_eq!(visible.len(), 1);
            assert_eq!(visible[0].name, expected);
        }
        let mut table = ProjectTable {
            filter: "no match".into(),
            ..Default::default()
        };
        assert!(table.visible_projects(&projects()).is_empty());
        table.filter = "  ".into();
        assert_eq!(table.visible_projects(&projects()).len(), 3);
    }
}
