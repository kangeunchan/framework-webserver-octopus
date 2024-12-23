/**
 * 이 코드에서는 두 개의 절차적 매크로(proc-macro)를 정의하고 있습니다:
 * - Controller: 특정 경로와 메서드를 매핑하여 라우터에 등록
 * - GetMapping: GetMapping 어트리뷰트를 처리하여 메서드를 라우터에 추가
 *
 * 각 부분을 하나씩 설명합니다.
 */

 use proc_macro::TokenStream; 
 use quote::quote; 
 use syn::{parse_macro_input, ItemTrait, TraitItem, parse::Parse, parse::ParseStream, LitStr}; 
 
 /**
  * ControllerPath 구조체 정의
  * `Controller` 매크로의 입력 값으로 전달된 경로를 파싱하기 위해 사용됨
  */
 struct ControllerPath {
     path: LitStr, // 경로는 LitStr(문자열 리터럴)로 저장됨
 }
 
 /**
  * ControllerPath 구조체의 Parse 트레이트 구현
  * 이 구현은 `Controller` 매크로에서 입력된 스트림을 파싱하여 ControllerPath 구조체로 변환
  */
 impl Parse for ControllerPath {
     fn parse(input: ParseStream) -> syn::Result<Self> {
         /*
          * 입력 스트림에서 경로를 파싱하여 ControllerPath 구조체를 반환
          */
         Ok(ControllerPath {
             path: input.parse()?, // 경로를 파싱
         })
     }
 }
 
 /**
  * Controller 매크로 정의
  * 이 매크로는 메서드들을 라우터에 추가하고, 경로를 등록하는 역할을 한다
  */
 #[proc_macro_attribute]
 pub fn Controller(attr: TokenStream, item: TokenStream) -> TokenStream {
     /*
      * `Controller` 매크로에서 전달된 인수인 `attr`을 `ControllerPath`로 파싱
      */
     let ctrl_path = parse_macro_input!(attr as ControllerPath);
     let base_path = ctrl_path.path.value(); // 경로를 문자열로 가져옴
     let input = parse_macro_input!(item as ItemTrait); // 전달된 trait 정의를 파싱
     let trait_name = &input.ident; // trait의 이름을 가져옴
 
     /*
      * 경로가 등록되는 부분을 추적하기 위한 출력문
      */
     println!("Controller base path: {}", base_path);
 
     /*
      * trait의 각 메서드를 검사하여, GetMapping 어트리뷰트를 찾고 이를 라우터에 추가
      */
     let route_implementations = input.items.iter().filter_map(|item| {
         if let TraitItem::Fn(method) = item {
             let method_name = &method.sig.ident; // 메서드 이름을 가져옴
             let get_mapping = method.attrs.iter().find(|attr| {
                 attr.path().is_ident("GetMapping") // GetMapping 어트리뷰트를 찾음
             });
 
             if let Some(mapping) = get_mapping {
                 if let Ok(meta) = mapping.parse_args::<LitStr>() { // GetMapping의 인수(경로)를 LitStr로 파싱
                     let path = meta.value(); // 경로 값 추출
                     let full_path = format!("{}{}", base_path, path); // 전체 경로(기본 경로 + 메서드 경로)
 
                     /*
                      * 등록된 경로 출력
                      */
                     println!("Registering route: {}", full_path);
 
                     /*
                      * route_implementations에 라우터에 추가할 코드 생성
                      */
                     Some(quote! {
                         router.add_route(#full_path.to_string(), || {
                             println!("Handler for: {}", #full_path); // 핸들러 호출 전 출력
                             <() as #trait_name>::#method_name(); // trait의 메서드를 호출
                         });
                     })
                 } else {
                     None
                 }
             } else {
                 None
             }
         } else {
             None
         }
     }).collect::<Vec<_>>(); // 모든 메서드를 처리하여 결과를 벡터에 수집
 
     /*
      * 확장된 코드 생성
      */
     let expanded = quote! {
         #input // 기존의 trait 정의를 그대로 포함
 
         /**
          * Router 구조체 정의
          * 경로와 핸들러를 저장
          */
         pub struct Router {
             routes: std::collections::HashMap<String, Box<dyn Fn() + Send + Sync>>, // 경로와 핸들러를 저장
         }
 
         impl Router {
             /**
              * Router 구조체의 새 인스턴스를 생성하는 함수
              */
             pub fn new() -> Self {
                 Self {
                     routes: std::collections::HashMap::new(), // 라우터의 경로를 담을 HashMap 초기화
                 }
             }
 
             /**
              * 라우터에 경로와 핸들러를 추가하는 함수
              */
             pub fn add_route<F>(&mut self, path: String, handler: F)
             where
                 F: Fn() + Send + Sync + 'static, // 핸들러는 Fn trait을 구현해야 함
             {
                 println!("Adding route: {}", path); // 경로가 라우터에 추가될 때마다 출력
                 self.routes.insert(path, Box::new(handler)); // HashMap에 경로와 핸들러 추가
             }
 
             /**
              * 요청을 처리하는 함수
              */
             pub fn handle_request(&self, path: &str) {
                 println!("Handling request for: {}", path); // 요청 처리 시작 시 출력
 
                 if let Some(handler) = self.routes.get(path) {
                     handler(); // 핸들러 실행
                 } else {
                     println!("404 Not Found: {}", path); // 경로가 없으면 404 출력
                 }
             }
         }
 
         /**
          * 라우터를 설정하는 함수
          */
         pub fn setup_router() -> Router {
             println!("Setting up router..."); // 라우터 설정 시작 시 출력
             let mut router = Router::new(); // 새로운 라우터 인스턴스 생성
             #(#route_implementations)* // 라우터에 메서드 경로를 추가하는 코드 삽입
             println!("Router setup complete."); // 라우터 설정 완료 시 출력
             router // 설정된 라우터 반환
         }
     };
 
     TokenStream::from(expanded) // 생성된 코드 반환
 }
 
 /**
  * GetMapping 매크로 정의
  * 이 매크로는 단순히 GetMapping 어트리뷰트를 메서드에 붙이기 위한 역할
  */
 #[proc_macro_attribute]
 pub fn GetMapping(attr: TokenStream, item: TokenStream) -> TokenStream {
     item // 매크로의 인수로 받은 item을 그대로 반환
 }
 