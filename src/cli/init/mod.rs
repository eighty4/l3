use crate::code::source::Language;
use crate::config::{is_valid_project_name, suggested_project_name_from_directory};
use crate::ui::prompt::choice::prompt_for_choice;
use crate::ui::prompt::input::{prompt_for_input, InputPromptConfig, ValidationResult};
use anyhow::anyhow;
use crossterm::style::Stylize;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod init_test;

const PRINT_PREFIX: &str = "  ";

pub struct InitOptions {
    pub language: Option<Language>,
    pub project_dir: PathBuf,
    pub project_name: Option<String>,
}

// todo support language/runtime specific init options (like npm vs pnpm)
// todo verify aws credentials and required roles after creating project
pub(crate) fn init_project(init_opts: InitOptions) -> Result<(), anyhow::Error> {
    let config_path = init_opts.project_dir.join("l3.yml");
    let data_dir = init_opts.project_dir.join(".l3");
    let routes_dir = init_opts.project_dir.join("routes");
    if config_path.exists() || data_dir.exists() || routes_dir.exists() {
        return Err(anyhow!("current directory already contains an L3 project"));
    }

    println!("λλλ init\n");
    let project_name = match init_opts.project_name {
        None => prompt_for_project_name(&init_opts.project_dir),
        Some(project_name) => {
            println!("{PRINT_PREFIX}Creating project `{project_name}` in ./\n");
            project_name
        }
    };

    let language = init_opts.language.unwrap_or_else(prompt_for_language);

    update_gitignore(&language, &init_opts.project_dir)?;
    create_config_file(&project_name, &init_opts.project_dir)?;
    create_routes_dir(&routes_dir)?;
    create_lambda_fn(&language, &routes_dir)?;

    println!("\n{PRINT_PREFIX}{PRINT_PREFIX}run `l3 sync` to deploy to AWS\n");
    Ok(())
}

fn prompt_for_project_name(project_dir: &Path) -> String {
    prompt_for_input(InputPromptConfig {
        _autocomplete_value: Some(suggested_project_name_from_directory(project_dir).as_str()),
        line_padding: PRINT_PREFIX,
        help_text: Some("AWS resources will be named with this"),
        prompt_text: "What is your project name?",
        validation: Some(|project_name| {
            if is_valid_project_name(project_name) {
                ValidationResult::Valid
            } else {
                ValidationResult::Invalid(
                    "Project name must only use alphanumeric, dash and underscore characters."
                        .to_string(),
                )
            }
        }),
    })
}

fn prompt_for_language() -> Language {
    let choice = prompt_for_choice(
        PRINT_PREFIX,
        "\nWhat language are you starting with?   (L3 projects can combine any of these languages after init)",
        vec![
            format!("{}", Language::JavaScript),
            format!("{}", Language::Python),
            format!("{}", Language::TypeScript),
        ],
    );
    println!();
    match choice.as_str() {
        "JavaScript" => Language::JavaScript,
        "Python" => Language::Python,
        "TypeScript" => Language::TypeScript,
        _ => panic!(),
    }
}

fn create_config_file(project_name: &str, project_dir: &Path) -> Result<(), anyhow::Error> {
    fs::write(
        project_dir.join("l3.yml"),
        format!("project_name: {project_name}\n"),
    )?;
    print_task_completed("created l3.yml config");
    Ok(())
}

fn create_routes_dir(routes_dir: &Path) -> Result<(), anyhow::Error> {
    fs::create_dir(routes_dir)?;
    print_task_completed("created routes directory for HTTP lambdas");
    Ok(())
}

fn update_gitignore(language: &Language, project_dir: &Path) -> Result<(), anyhow::Error> {
    let gitignore_entries = match language {
        Language::JavaScript | Language::TypeScript => vec![".l3", "node_modules"],
        Language::Python => vec![".l3"],
    };
    let gitignore = project_dir.join(".gitignore");
    if gitignore.exists() {
        let gitignore_content = fs::read_to_string(&gitignore)?;
        let existing_entries: Vec<String> =
            gitignore_content.lines().map(|s| s.to_string()).collect();
        let mut to_append: Vec<String> = Vec::with_capacity(gitignore_entries.len());
        for gitignore_entry in gitignore_entries {
            let s = String::from(gitignore_entry);
            if !existing_entries.contains(&s) {
                to_append.push(s);
            }
        }
        if !to_append.is_empty() {
            let write_content = if gitignore_content.ends_with("\n") {
                to_append.join("\n")
            } else {
                format!("\n{}", to_append.join("\n"))
            };
            let mut f = fs::OpenOptions::new().append(true).open(&gitignore)?;
            f.write_all(write_content.as_bytes())?;
            print_task_completed("updated .gitignore");
        }
    } else {
        fs::write(gitignore, gitignore_entries.join("\n"))?;
        print_task_completed("created .gitignore");
    }
    Ok(())
}

fn create_lambda_fn(language: &Language, routes_dir: &Path) -> Result<(), anyhow::Error> {
    let lambda_file_name = match language {
        Language::JavaScript => "lambda.js",
        Language::Python => "lambda.py",
        Language::TypeScript => "lambda.ts",
    };
    let lambda_content = match language {
        Language::JavaScript | Language::TypeScript => include_str!("[gen]lambda.js"),
        Language::Python => include_str!("[gen]lambda.py"),
    };
    fs::write(routes_dir.join(lambda_file_name), lambda_content)?;
    print_task_completed(format!("created Lambda at routes/{lambda_file_name}").as_str());
    Ok(())
}

fn print_task_completed(msg: &str) {
    println!("{PRINT_PREFIX}{} {msg}", "✔".green());
}
