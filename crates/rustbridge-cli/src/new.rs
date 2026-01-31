//! New project command implementation
//!
//! Generates rustbridge plugin projects with optional consumer projects for
//! various host languages (Kotlin, Java, C#, Python).

use anyhow::Result;
use std::fs;
use std::path::Path;

// ============================================================================
// Embedded Templates
// ============================================================================

mod templates {
    // Rust plugin templates
    pub const RUST_CARGO_TOML: &str = include_str!("../templates/rust/Cargo.toml.tmpl");
    pub const RUST_LIB_RS: &str = include_str!("../templates/rust/src/lib.rs.tmpl");
    pub const RUST_GITIGNORE: &str = include_str!("../templates/rust/.gitignore");

    // Kotlin consumer templates
    pub const KOTLIN_BUILD_GRADLE: &str = include_str!("../templates/kotlin/build.gradle.kts");
    pub const KOTLIN_SETTINGS_GRADLE: &str =
        include_str!("../templates/kotlin/settings.gradle.kts.tmpl");
    pub const KOTLIN_GRADLE_PROPERTIES: &str =
        include_str!("../templates/kotlin/gradle.properties");
    pub const KOTLIN_MAIN: &str =
        include_str!("../templates/kotlin/src/main/kotlin/com/example/Main.kt.tmpl");
    pub const KOTLIN_GITIGNORE: &str = include_str!("../templates/kotlin/.gitignore");
    pub const KOTLIN_GRADLEW: &str = include_str!("../templates/kotlin/gradlew");
    pub const KOTLIN_GRADLEW_BAT: &str = include_str!("../templates/kotlin/gradlew.bat");
    pub const KOTLIN_GRADLE_WRAPPER_PROPERTIES: &str =
        include_str!("../templates/kotlin/gradle/wrapper/gradle-wrapper.properties");
    pub const KOTLIN_GRADLE_WRAPPER_JAR: &[u8] =
        include_bytes!("../templates/kotlin/gradle/wrapper/gradle-wrapper.jar");

    // Java FFM consumer templates
    pub const JAVA_FFM_BUILD_GRADLE: &str = include_str!("../templates/java-ffm/build.gradle.kts");
    pub const JAVA_FFM_SETTINGS_GRADLE: &str =
        include_str!("../templates/java-ffm/settings.gradle.kts.tmpl");
    pub const JAVA_FFM_MAIN: &str =
        include_str!("../templates/java-ffm/src/main/java/com/example/Main.java.tmpl");
    pub const JAVA_FFM_GITIGNORE: &str = include_str!("../templates/java-ffm/.gitignore");
    pub const JAVA_FFM_GRADLEW: &str = include_str!("../templates/java-ffm/gradlew");
    pub const JAVA_FFM_GRADLEW_BAT: &str = include_str!("../templates/java-ffm/gradlew.bat");
    pub const JAVA_FFM_GRADLE_WRAPPER_PROPERTIES: &str =
        include_str!("../templates/java-ffm/gradle/wrapper/gradle-wrapper.properties");
    pub const JAVA_FFM_GRADLE_WRAPPER_JAR: &[u8] =
        include_bytes!("../templates/java-ffm/gradle/wrapper/gradle-wrapper.jar");

    // Java JNI consumer templates
    pub const JAVA_JNI_BUILD_GRADLE: &str = include_str!("../templates/java-jni/build.gradle.kts");
    pub const JAVA_JNI_SETTINGS_GRADLE: &str =
        include_str!("../templates/java-jni/settings.gradle.kts.tmpl");
    pub const JAVA_JNI_MAIN: &str =
        include_str!("../templates/java-jni/src/main/java/com/example/Main.java.tmpl");
    pub const JAVA_JNI_GITIGNORE: &str = include_str!("../templates/java-jni/.gitignore");
    pub const JAVA_JNI_GRADLEW: &str = include_str!("../templates/java-jni/gradlew");
    pub const JAVA_JNI_GRADLEW_BAT: &str = include_str!("../templates/java-jni/gradlew.bat");
    pub const JAVA_JNI_GRADLE_WRAPPER_PROPERTIES: &str =
        include_str!("../templates/java-jni/gradle/wrapper/gradle-wrapper.properties");
    pub const JAVA_JNI_GRADLE_WRAPPER_JAR: &[u8] =
        include_bytes!("../templates/java-jni/gradle/wrapper/gradle-wrapper.jar");

    // C# consumer templates
    pub const CSHARP_CSPROJ: &str = include_str!("../templates/csharp/Consumer.csproj.tmpl");
    pub const CSHARP_PROGRAM: &str = include_str!("../templates/csharp/Program.cs.tmpl");
    pub const CSHARP_GITIGNORE: &str = include_str!("../templates/csharp/.gitignore");

    // Python consumer templates
    pub const PYTHON_MAIN: &str = include_str!("../templates/python/main.py.tmpl");
    pub const PYTHON_REQUIREMENTS: &str = include_str!("../templates/python/requirements.txt");
    pub const PYTHON_GITIGNORE: &str = include_str!("../templates/python/.gitignore");
}

// ============================================================================
// Template Context
// ============================================================================

/// Context for template variable substitution
struct TemplateContext {
    /// Project name with dashes (e.g., "my-plugin")
    project_name: String,
    /// PascalCase class name (e.g., "MyPlugin")
    class_name: String,
    /// Snake_case package name for Rust/Python (e.g., "my_plugin")
    package_name: String,
    /// Default bundle path (e.g., "my-plugin-0.1.0.rbp")
    bundle_path: String,
}

impl TemplateContext {
    fn new(name: &str) -> Self {
        Self {
            project_name: name.to_string(),
            class_name: to_pascal_case(name),
            package_name: name.replace('-', "_"),
            bundle_path: format!("{name}-0.1.0.rbp"),
        }
    }

    /// Apply placeholder substitutions to template content
    fn apply(&self, template: &str) -> String {
        template
            .replace("{{project-name}}", &self.project_name)
            .replace("{{class-name}}", &self.class_name)
            .replace("{{package-name}}", &self.package_name)
            .replace("{{bundle-path}}", &self.bundle_path)
    }
}

// ============================================================================
// Options
// ============================================================================

/// Options for the new command
#[derive(Default)]
pub struct NewOptions {
    pub kotlin: bool,
    pub java_ffm: bool,
    pub java_jni: bool,
    pub csharp: bool,
    pub python: bool,
}

impl NewOptions {
    /// Returns true if any consumer language is selected
    fn has_consumers(&self) -> bool {
        self.kotlin || self.java_ffm || self.java_jni || self.csharp || self.python
    }
}

// ============================================================================
// Main Entry Point
// ============================================================================

/// Run the new command
pub fn run(name: &str, path: Option<String>, options: NewOptions) -> Result<()> {
    let project_dir = path.unwrap_or_else(|| name.to_string());
    let project_path = Path::new(&project_dir);
    let ctx = TemplateContext::new(name);

    println!("Creating new rustbridge plugin: {name}");
    println!("Directory: {project_dir}");

    // Check if directory already exists
    if project_path.exists() {
        anyhow::bail!("Directory already exists: {project_dir}");
    }

    // Create Rust plugin at root
    create_rust_plugin(project_path, &ctx)?;

    // Create consumers/ subdirectory if any consumer is requested
    if options.has_consumers() {
        let consumers_dir = project_path.join("consumers");

        if options.kotlin {
            create_kotlin_consumer(&consumers_dir, &ctx)?;
        }
        if options.java_ffm {
            create_java_ffm_consumer(&consumers_dir, &ctx)?;
        }
        if options.java_jni {
            create_java_jni_consumer(&consumers_dir, &ctx)?;
        }
        if options.csharp {
            create_csharp_consumer(&consumers_dir, &ctx)?;
        }
        if options.python {
            create_python_consumer(&consumers_dir, &ctx)?;
        }
    }

    print_next_steps(&project_dir, &ctx, &options);
    Ok(())
}

// ============================================================================
// Rust Plugin Creation
// ============================================================================

fn create_rust_plugin(base: &Path, ctx: &TemplateContext) -> Result<()> {
    fs::create_dir_all(base.join("src"))?;

    fs::write(
        base.join("Cargo.toml"),
        ctx.apply(templates::RUST_CARGO_TOML),
    )?;
    fs::write(base.join("src/lib.rs"), ctx.apply(templates::RUST_LIB_RS))?;
    fs::write(base.join(".gitignore"), templates::RUST_GITIGNORE)?;

    println!("  Created Rust plugin");
    Ok(())
}

// ============================================================================
// Consumer Project Creation
// ============================================================================

fn create_kotlin_consumer(consumers_dir: &Path, ctx: &TemplateContext) -> Result<()> {
    let kotlin_dir = consumers_dir.join("kotlin");
    let src_dir = kotlin_dir.join("src/main/kotlin/com/example");
    let wrapper_dir = kotlin_dir.join("gradle/wrapper");

    fs::create_dir_all(&src_dir)?;
    fs::create_dir_all(&wrapper_dir)?;

    // Templated files
    fs::write(
        kotlin_dir.join("settings.gradle.kts"),
        ctx.apply(templates::KOTLIN_SETTINGS_GRADLE),
    )?;
    fs::write(src_dir.join("Main.kt"), ctx.apply(templates::KOTLIN_MAIN))?;

    // Static files
    fs::write(
        kotlin_dir.join("build.gradle.kts"),
        templates::KOTLIN_BUILD_GRADLE,
    )?;
    fs::write(
        kotlin_dir.join("gradle.properties"),
        templates::KOTLIN_GRADLE_PROPERTIES,
    )?;
    fs::write(kotlin_dir.join(".gitignore"), templates::KOTLIN_GITIGNORE)?;
    fs::write(kotlin_dir.join("gradlew"), templates::KOTLIN_GRADLEW)?;
    fs::write(
        kotlin_dir.join("gradlew.bat"),
        templates::KOTLIN_GRADLEW_BAT,
    )?;
    fs::write(
        wrapper_dir.join("gradle-wrapper.properties"),
        templates::KOTLIN_GRADLE_WRAPPER_PROPERTIES,
    )?;
    fs::write(
        wrapper_dir.join("gradle-wrapper.jar"),
        templates::KOTLIN_GRADLE_WRAPPER_JAR,
    )?;

    // Make gradlew executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(kotlin_dir.join("gradlew"))?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(kotlin_dir.join("gradlew"), perms)?;
    }

    println!("  Created consumers/kotlin");
    Ok(())
}

fn create_java_ffm_consumer(consumers_dir: &Path, ctx: &TemplateContext) -> Result<()> {
    let java_dir = consumers_dir.join("java-ffm");
    let src_dir = java_dir.join("src/main/java/com/example");
    let wrapper_dir = java_dir.join("gradle/wrapper");

    fs::create_dir_all(&src_dir)?;
    fs::create_dir_all(&wrapper_dir)?;

    // Templated files
    fs::write(
        java_dir.join("settings.gradle.kts"),
        ctx.apply(templates::JAVA_FFM_SETTINGS_GRADLE),
    )?;
    fs::write(
        src_dir.join("Main.java"),
        ctx.apply(templates::JAVA_FFM_MAIN),
    )?;

    // Static files
    fs::write(
        java_dir.join("build.gradle.kts"),
        templates::JAVA_FFM_BUILD_GRADLE,
    )?;
    fs::write(java_dir.join(".gitignore"), templates::JAVA_FFM_GITIGNORE)?;
    fs::write(java_dir.join("gradlew"), templates::JAVA_FFM_GRADLEW)?;
    fs::write(
        java_dir.join("gradlew.bat"),
        templates::JAVA_FFM_GRADLEW_BAT,
    )?;
    fs::write(
        wrapper_dir.join("gradle-wrapper.properties"),
        templates::JAVA_FFM_GRADLE_WRAPPER_PROPERTIES,
    )?;
    fs::write(
        wrapper_dir.join("gradle-wrapper.jar"),
        templates::JAVA_FFM_GRADLE_WRAPPER_JAR,
    )?;

    // Make gradlew executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(java_dir.join("gradlew"))?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(java_dir.join("gradlew"), perms)?;
    }

    println!("  Created consumers/java-ffm");
    Ok(())
}

fn create_java_jni_consumer(consumers_dir: &Path, ctx: &TemplateContext) -> Result<()> {
    let java_dir = consumers_dir.join("java-jni");
    let src_dir = java_dir.join("src/main/java/com/example");
    let wrapper_dir = java_dir.join("gradle/wrapper");

    fs::create_dir_all(&src_dir)?;
    fs::create_dir_all(&wrapper_dir)?;

    // Templated files
    fs::write(
        java_dir.join("settings.gradle.kts"),
        ctx.apply(templates::JAVA_JNI_SETTINGS_GRADLE),
    )?;
    fs::write(
        src_dir.join("Main.java"),
        ctx.apply(templates::JAVA_JNI_MAIN),
    )?;

    // Static files
    fs::write(
        java_dir.join("build.gradle.kts"),
        templates::JAVA_JNI_BUILD_GRADLE,
    )?;
    fs::write(java_dir.join(".gitignore"), templates::JAVA_JNI_GITIGNORE)?;
    fs::write(java_dir.join("gradlew"), templates::JAVA_JNI_GRADLEW)?;
    fs::write(
        java_dir.join("gradlew.bat"),
        templates::JAVA_JNI_GRADLEW_BAT,
    )?;
    fs::write(
        wrapper_dir.join("gradle-wrapper.properties"),
        templates::JAVA_JNI_GRADLE_WRAPPER_PROPERTIES,
    )?;
    fs::write(
        wrapper_dir.join("gradle-wrapper.jar"),
        templates::JAVA_JNI_GRADLE_WRAPPER_JAR,
    )?;

    // Make gradlew executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(java_dir.join("gradlew"))?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(java_dir.join("gradlew"), perms)?;
    }

    println!("  Created consumers/java-jni");
    Ok(())
}

fn create_csharp_consumer(consumers_dir: &Path, ctx: &TemplateContext) -> Result<()> {
    let csharp_dir = consumers_dir.join("csharp");
    fs::create_dir_all(&csharp_dir)?;

    // Templated files
    fs::write(
        csharp_dir.join(format!("{}.csproj", ctx.class_name)),
        ctx.apply(templates::CSHARP_CSPROJ),
    )?;
    fs::write(
        csharp_dir.join("Program.cs"),
        ctx.apply(templates::CSHARP_PROGRAM),
    )?;

    // Static files
    fs::write(csharp_dir.join(".gitignore"), templates::CSHARP_GITIGNORE)?;

    println!("  Created consumers/csharp");
    Ok(())
}

fn create_python_consumer(consumers_dir: &Path, ctx: &TemplateContext) -> Result<()> {
    let python_dir = consumers_dir.join("python");
    fs::create_dir_all(&python_dir)?;

    // Templated files
    fs::write(
        python_dir.join("main.py"),
        ctx.apply(templates::PYTHON_MAIN),
    )?;

    // Static files
    fs::write(
        python_dir.join("requirements.txt"),
        templates::PYTHON_REQUIREMENTS,
    )?;
    fs::write(python_dir.join(".gitignore"), templates::PYTHON_GITIGNORE)?;

    // Make main.py executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(python_dir.join("main.py"))?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(python_dir.join("main.py"), perms)?;
    }

    println!("  Created consumers/python");
    Ok(())
}

// ============================================================================
// Next Steps
// ============================================================================

fn print_next_steps(project_dir: &str, ctx: &TemplateContext, options: &NewOptions) {
    println!("\nProject created successfully!\n");
    println!("Next steps:");
    println!("  cd {project_dir}");
    println!("  cargo build --release");
    println!(
        "  rustbridge bundle create --name {} --version 0.1.0 \\",
        ctx.project_name
    );
    println!(
        "    --lib linux-x86_64:target/release/lib{}.so \\",
        ctx.package_name
    );
    println!("    --output {}", ctx.bundle_path);

    if options.kotlin {
        println!("\nKotlin consumer:");
        println!("  cd consumers/kotlin");
        println!("  cp ../../{} .", ctx.bundle_path);
        println!("  ./gradlew run");
    }

    if options.java_ffm {
        println!("\nJava FFM consumer (requires Java 21+):");
        println!("  cd consumers/java-ffm");
        println!("  cp ../../{} .", ctx.bundle_path);
        println!("  ./gradlew run");
    }

    if options.java_jni {
        println!("\nJava JNI consumer (requires Java 17+):");
        println!("  cd consumers/java-jni");
        println!("  cp ../../{} .", ctx.bundle_path);
        println!("  ./gradlew run");
    }

    if options.csharp {
        println!("\nC# consumer (requires .NET 8+):");
        println!("  cd consumers/csharp");
        println!("  cp ../../{} .", ctx.bundle_path);
        println!("  dotnet run");
    }

    if options.python {
        println!("\nPython consumer:");
        println!("  cd consumers/python");
        println!("  cp ../../{} .", ctx.bundle_path);
        println!("  pip install -r requirements.txt");
        println!("  python main.py");
    }
}

// ============================================================================
// Utilities
// ============================================================================

/// Convert a string to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split(['-', '_'])
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}
