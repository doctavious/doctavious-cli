# TIL

> Today I Learned

* Categories: {{categories_count}}
* TILs: {{til_count}}

## Categories

{% for key,value in tils -%}
* [{{key}}](#{{key}}) 
{% endfor %}

{%- for key,value in tils %}
## {{key}}
{% for v in value -%}
* [{{v.title}}]({{key}}/{{v.file_name}}) - {{v.date | date(format="%Y-%m-%d")}}
{% endfor %}
{%- endfor %}
