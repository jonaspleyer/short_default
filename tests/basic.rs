use short_default::default;

#[test]
fn struct_def() {
    default!(
        pub struct Settings {
            threads: usize = 1,
            name: String = "Short Default".to_string(),
        }
    );
    let settings = Settings {
        threads: 2,
        name: "This is nice".to_string(),
    };
    assert_eq!(settings.threads, 2);
    assert_eq!(settings.name, "This is nice");
}

#[test]
fn default_values() {
    default!(
        pub struct Config {
            buffer_size: u16 = 10,
            initial_values: u8 = 1,
        }
    );
    let config = Config::default();
    assert_eq!(config.buffer_size, 10);
    assert_eq!(config.initial_values, 1);
}

#[test]
fn default_with_blanks() {
    default!(
        struct Person {
            name: String,
        }
    );
    let person = Person::default();
    assert_eq!(person.name, "");
}

#[test]
fn generics() {
    default!(
        struct MyVec<T> {
            inner: Vec<T>,
        }
    );
    let myvec = MyVec::<i32>::default();
    assert_eq!(myvec.inner.len(), 0);
}

#[test]
fn field_attributes() {
    use approx_derive::AbsDiffEq;

    default! {
        #[derive(Clone, Debug, AbsDiffEq, PartialEq)]
        struct Chili {
            /// This should be on a scale of 0.0 to 10.0
            hunger: f32 = 9.0,
        }
    }
    let chili = Chili::default();
    let chili2 = Chili { hunger: 8.5 };
    assert_eq!(chili.hunger, 9.0);
    approx::assert_abs_diff_eq!(chili, chili2, epsilon = 0.6);
    approx::assert_abs_diff_ne!(chili, chili2);
}
