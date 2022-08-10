use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Options {
    pub factory_converters: Vec<(String, usize)>,
}

impl Options {
    pub fn new(factory_converters: Vec<(String, usize)>) -> Self {
        Self { factory_converters }
    }

    pub fn render_impl(&self, tera: &tera::Tera) -> tera::Result<String> {
        let mut context = tera::Context::new();
        context.insert("options", self);
        tera.render("options_impl.cs", &context)
    }

    pub fn into_factories(self) -> Vec<Factory> {
        self.factory_converters
            .into_iter()
            .map(|(name, count)| Factory::new(name, count))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Factory {
    pub name: String,
    generic_commas: String,
}

impl Factory {
    pub fn new(name: String, count: usize) -> Self {
        let generic_commas = ",".repeat(count.saturating_sub(1));
        Self {
            name,
            generic_commas,
        }
    }

    pub fn render_impl(&self, tera: &tera::Tera) -> tera::Result<String> {
        let mut context = tera::Context::new();
        context.insert("factory", self);
        tera.render("converter_factory.cs", &context)
    }
}

pub const OPTIONS_IMPL: &'static str = r###"using System.Collections.Generic;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Mech3DotNet.Json.Converters
{
    public static partial class Options
    {
        public static List<JsonConverter> GetDefaultConverters() => new List<JsonConverter>
        {
{%- for converter in options.factory_converters %}
            new {{ converter.0 }}ConverterFactory(),
{%- endfor %}
        };
    }
}
"###;

pub const CONV_FACTORY: &'static str = r###"using System;
using System.Reflection;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Mech3DotNet.Json.Converters
{
    public class {{ factory.name }}ConverterFactory : JsonConverterFactory
    {
        public override bool CanConvert(Type typeToConvert)
        {
            if (!typeToConvert.IsGenericType)
                return false;
            if (typeToConvert.GetGenericTypeDefinition() != typeof({{ factory.name }}<{{ factory.generic_commas }}>))
                return false;
            return true;
        }

        public override JsonConverter CreateConverter(Type type, JsonSerializerOptions options)
        {
            return (JsonConverter)Activator.CreateInstance(
                typeof({{ factory.name }}Converter<{{ factory.generic_commas }}>).MakeGenericType(type.GetGenericArguments()),
                BindingFlags.Instance | BindingFlags.Public,
                binder: null,
                args: new object[] { options },
                culture: null)!;
        }
    }
}
"###;
