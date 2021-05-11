use crate::domain::model::operation::OperationRepository;
use crate::domain::model::route::{Route, RouteEditor, RouteRepository};
use crate::domain::model::types::RouteId;
use crate::utils::error::ApplicationResult;

pub struct RouteService<R: RouteRepository, O: OperationRepository> {
    route_repository: R,
    operation_repository: O,
}

impl<R: RouteRepository, O: OperationRepository> RouteService<R, O> {
    pub fn new(route_repository: R, operation_repository: O) -> Self {
        Self {
            route_repository,
            operation_repository,
        }
    }

    pub fn find_route(&self, route_id: &RouteId) -> ApplicationResult<Route> {
        self.route_repository.find(route_id)
    }

    pub fn find_all_routes(&self) -> ApplicationResult<Vec<Route>> {
        self.route_repository.find_all()
    }

    pub fn find_editor(&self, route_id: &RouteId) -> ApplicationResult<RouteEditor> {
        let route = self.find_route(route_id)?;
        let operations = self.operation_repository.find_by_route_id(route_id)?;

        Ok(RouteEditor::new(route, operations))
    }

    pub fn update_route(&self, route: &Route) -> ApplicationResult<()> {
        self.route_repository.update(route)
    }

    pub fn update_editor(&self, editor: &RouteEditor) -> ApplicationResult<()> {
        self.update_route(editor.route())?;
        self.operation_repository
            .update_by_route_id(editor.route().id(), editor.op_list())
    }

    pub fn register_route(&self, route: &Route) -> ApplicationResult<()> {
        self.route_repository.register(route)
    }

    pub fn register_editor(&self, editor: &RouteEditor) -> ApplicationResult<()> {
        self.route_repository.register(editor.route())?;
        self.operation_repository
            .update_by_route_id(editor.route().id(), editor.op_list())
    }

    pub fn delete_editor(&self, route_id: &RouteId) -> ApplicationResult<()> {
        self.route_repository.delete(route_id)?;
        self.operation_repository.delete_by_route_id(route_id)
    }
}